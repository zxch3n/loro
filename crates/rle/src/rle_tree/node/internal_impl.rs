use std::{
    collections::HashSet,
    fmt::{Debug, Error, Formatter},
};

use crate::rle_tree::tree_trait::{FindPosResult, Position};

use super::*;

impl<'a, T: Rle, A: RleTreeTrait<T>> InternalNode<'a, T, A> {
    pub fn new(bump: &'a Bump, parent: Option<NonNull<Self>>) -> Self {
        Self {
            bump,
            parent,
            children: BumpVec::with_capacity_in(A::MAX_CHILDREN_NUM, bump),
            cache: Default::default(),
            _pin: PhantomPinned,
            _a: PhantomData,
        }
    }

    #[inline]
    fn _split(&mut self) -> &'a mut Node<'a, T, A> {
        let ans = self
            .bump
            .alloc(Node::Internal(Self::new(self.bump, self.parent)));
        let inner_ptr = NonNull::new(&mut *ans.as_internal_mut().unwrap()).unwrap();
        let inner = ans.as_internal_mut().unwrap();
        for child in self
            .children
            .drain(self.children.len() - A::MIN_CHILDREN_NUM..self.children.len())
        {
            child.set_parent(inner_ptr);
            inner.children.push(child);
        }

        ans
    }

    #[inline]
    pub fn children(&self) -> &[&'a mut Node<'a, T, A>] {
        &self.children
    }

    #[cfg(test)]
    pub(crate) fn _check_child_parent(&self) {
        for child in self.children.iter() {
            child.get_self_index().unwrap();
            match child {
                Node::Internal(node) => {
                    assert!(std::ptr::eq(node.parent.unwrap().as_ptr(), self));
                    node._check_child_parent();
                }
                Node::Leaf(node) => {
                    assert!(std::ptr::eq(node.parent.as_ptr(), self));
                }
            }
        }
    }

    pub(crate) fn check(&mut self) {
        if !self.is_root() {
            assert!(
                self.children.len() >= A::MIN_CHILDREN_NUM,
                "children.len() = {}",
                self.children.len()
            );
            assert!(
                self.children.len() <= A::MAX_CHILDREN_NUM,
                "children.len() = {}",
                self.children.len()
            );
        }

        let self_ptr = self as *const _;
        for child in self.children.iter_mut() {
            match child {
                Node::Internal(node) => {
                    node.check();
                    assert!(std::ptr::eq(node.parent.unwrap().as_ptr(), self_ptr));
                }
                Node::Leaf(node) => {
                    node.check();
                    assert!(std::ptr::eq(node.parent.as_ptr(), self_ptr));
                }
            }
        }

        A::check_cache_internal(self);
    }

    // TODO: simplify this func?
    fn _delete<F>(
        &mut self,
        from: Option<A::Int>,
        to: Option<A::Int>,
        visited: &mut Vec<(usize, NonNull<Node<'a, T, A>>)>,
        depth: usize,
        notify: &mut F,
    ) -> Result<(), &'a mut Node<'a, T, A>>
    where
        F: FnMut(&T, *mut LeafNode<'_, T, A>),
    {
        if self.children.is_empty() {
            return Ok(());
        }

        let (direct_delete_start, to_del_start_offset) =
            from.map_or((0, None), |x| self._delete_start(x));
        let (direct_delete_end, to_del_end_offset) =
            to.map_or((self.children.len(), None), |x| self._delete_end(x));
        let mut result = Ok(());
        {
            // handle edge removing
            let mut handled = false;
            if let (Some(del_from), Some(del_to)) = (to_del_start_offset, to_del_end_offset) {
                if direct_delete_start - 1 == direct_delete_end {
                    visited.push((
                        depth,
                        NonNull::new(&mut *self.children[direct_delete_end]).unwrap(),
                    ));
                    match &mut self.children[direct_delete_end] {
                        Node::Internal(node) => {
                            if let Err(new) = node._delete(
                                Some(del_from),
                                Some(del_to),
                                visited,
                                depth + 1,
                                notify,
                            ) {
                                result = self._insert_with_split(direct_delete_end + 1, new);
                            }
                        }
                        Node::Leaf(node) => {
                            if let Err(new) = node.delete(Some(del_from), Some(del_to), notify) {
                                result = self._insert_with_split(direct_delete_end + 1, new);
                            }
                        }
                    }
                    handled = true;
                }
            }

            if !handled {
                if let Some(del_from) = to_del_start_offset {
                    visited.push((
                        depth,
                        NonNull::new(&mut *self.children[direct_delete_start - 1]).unwrap(),
                    ));
                    match &mut self.children[direct_delete_start - 1] {
                        Node::Internal(node) => {
                            if let Err(new) =
                                node._delete(Some(del_from), None, visited, depth + 1, notify)
                            {
                                result = self._insert_with_split(direct_delete_start, new);
                            }
                        }
                        Node::Leaf(node) => {
                            if let Err(new) = node.delete(Some(del_from), None, notify) {
                                result = self._insert_with_split(direct_delete_start, new)
                            }
                        }
                    }
                }
                if let Some(del_to) = to_del_end_offset {
                    visited.push((
                        depth,
                        NonNull::new(&mut *self.children[direct_delete_end]).unwrap(),
                    ));
                    match &mut self.children[direct_delete_end] {
                        Node::Internal(node) => {
                            if let Err(new) =
                                node._delete(None, Some(del_to), visited, depth + 1, notify)
                            {
                                debug_assert!(result.is_ok());
                                result = self._insert_with_split(direct_delete_end + 1, new);
                            }
                        }
                        Node::Leaf(node) => {
                            if let Err(new) = node.delete(None, Some(del_to), notify) {
                                debug_assert!(result.is_ok());
                                result = self._insert_with_split(direct_delete_end + 1, new);
                            }
                        }
                    }
                }
            }
        }

        if direct_delete_start < direct_delete_end {
            self.children.drain(direct_delete_start..direct_delete_end);
        }

        A::update_cache_internal(self);
        if let Err(new) = &mut result {
            A::update_cache_internal(new.as_internal_mut().unwrap());
        }

        result
    }

    fn _delete_start(&mut self, from: A::Int) -> (usize, Option<A::Int>) {
        let from = A::find_pos_internal(self, from);
        if from.pos == Position::Start || from.pos == Position::Before {
            (from.child_index, None)
        } else {
            (from.child_index + 1, Some(from.offset))
        }
    }

    fn _delete_end(&mut self, to: A::Int) -> (usize, Option<A::Int>) {
        let to = A::find_pos_internal(self, to);
        if to.pos == Position::End || to.pos == Position::After {
            (to.child_index + 1, None)
        } else {
            (to.child_index, Some(to.offset))
        }
    }

    pub fn insert<F>(
        &mut self,
        index: A::Int,
        value: T,
        notify: &mut F,
    ) -> Result<(), &'a mut Node<'a, T, A>>
    where
        F: FnMut(&T, *mut LeafNode<'_, T, A>),
    {
        match self._insert(index, value, notify) {
            Ok(_) => {
                A::update_cache_internal(self);
                Ok(())
            }
            Err(new) => {
                A::update_cache_internal(self);
                A::update_cache_internal(new.as_internal_mut().unwrap());
                if self.is_root() {
                    self._create_level(new);
                    Ok(())
                } else {
                    Err(new)
                }
            }
        }
    }

    /// root node function. assume self and new's caches are up-to-date
    fn _create_level(&mut self, new: &'a mut Node<'a, T, A>) {
        debug_assert!(self.is_root());
        let left = self
            .bump
            .alloc(Node::Internal(InternalNode::new(self.bump, None)));
        let left_inner = left.as_internal_mut().unwrap();
        std::mem::swap(left_inner, self);
        let left_ptr = left_inner.into();
        for child in left_inner.children.iter_mut() {
            child.set_parent(left_ptr);
        }

        left_inner.parent = Some(NonNull::new(self).unwrap());
        new.as_internal_mut().unwrap().parent = Some(NonNull::new(self).unwrap());
        self.children.push(left);
        self.children.push(new);
        A::update_cache_internal(self);
    }

    fn _insert<F>(
        &mut self,
        index: A::Int,
        value: T,
        notify: &mut F,
    ) -> Result<(), &'a mut Node<'a, T, A>>
    where
        F: FnMut(&T, *mut LeafNode<'_, T, A>),
    {
        if self.children.is_empty() {
            debug_assert!(self.is_root());
            let ptr = NonNull::new(self as *mut _).unwrap();
            self.children.push(Node::new_leaf(self.bump, ptr));
        }

        let FindPosResult {
            child_index,
            offset: relative_idx,
            ..
        } = A::find_pos_internal(self, index);
        let child = &mut self.children[child_index];
        let new = match child {
            Node::Internal(child) => child.insert(relative_idx, value, notify),
            Node::Leaf(child) => child.insert(relative_idx, value, notify),
        };

        if let Err(new) = new {
            if let Err(value) = self._insert_with_split(child_index + 1, new) {
                return Err(value);
            }
        }

        Ok(())
    }
}

impl<'a, T: Rle, A: RleTreeTrait<T>> InternalNode<'a, T, A> {
    /// this can only invoke from root
    #[inline]
    pub(crate) fn delete<F>(&mut self, start: Option<A::Int>, end: Option<A::Int>, notify: &mut F)
    where
        F: FnMut(&T, *mut LeafNode<'_, T, A>),
    {
        debug_assert!(self.is_root());
        let mut zipper = Vec::new();
        match self._delete(start, end, &mut zipper, 1, notify) {
            Ok(_) => {
                A::update_cache_internal(self);
            }
            Err(new) => {
                A::update_cache_internal(self);
                A::update_cache_internal(new.as_internal_mut().unwrap());
                self._create_level(new);
            }
        };

        let removed = self._root_shrink_levels_if_one_child();

        // filter the same
        let mut visited: HashSet<NonNull<_>> = HashSet::default();
        let mut should_skip: HashSet<NonNull<_>> = HashSet::default();
        let mut zipper: Vec<(usize, NonNull<Node<'a, T, A>>)> = zipper
            .into_iter()
            .filter(|(_, ptr)| {
                if visited.contains(ptr) {
                    false
                } else {
                    visited.insert(*ptr);
                    true
                }
            })
            .collect();
        // visit in depth order, top to down (depth 0..inf)
        zipper.sort();
        let mut any_delete: bool;
        loop {
            any_delete = false;
            for (_, mut node_ptr) in zipper.iter() {
                if should_skip.contains(&node_ptr) {
                    continue;
                }

                let node = unsafe { node_ptr.as_mut() };
                if let Some(node) = node.as_internal() {
                    let ptr = node as *const InternalNode<'a, T, A>;
                    if removed.contains(&ptr) {
                        should_skip.insert(node_ptr);
                        continue;
                    }
                }

                debug_assert!(node.children_num() <= A::MAX_CHILDREN_NUM);
                if node.children_num() >= A::MIN_CHILDREN_NUM {
                    continue;
                }

                let mut to_delete: bool = false;
                if let Some((sibling, either)) = node.get_a_sibling() {
                    // if has sibling, borrow or merge to it
                    let sibling: &mut Node<'a, T, A> =
                        unsafe { &mut *((sibling as *const _) as usize as *mut _) };
                    if node.children_num() + sibling.children_num() <= A::MAX_CHILDREN_NUM {
                        node.merge_to_sibling(sibling, either, notify);
                        to_delete = true;
                    } else {
                        node.borrow_from_sibling(sibling, either, notify);
                    }
                } else {
                    if node.parent().unwrap().is_root() {
                        continue;
                    }

                    dbg!(self);
                    dbg!(node.parent());
                    dbg!(node);
                    unreachable!();
                }

                if to_delete {
                    should_skip.insert(node_ptr);
                    any_delete = true;
                    node.remove();
                }
            }

            if !any_delete {
                break;
            }
        }

        self._root_shrink_levels_if_one_child();
    }

    fn _root_shrink_levels_if_one_child(&mut self) -> HashSet<*const InternalNode<'a, T, A>> {
        let mut ans: HashSet<_> = Default::default();
        while self.children.len() == 1 && self.children[0].as_internal().is_some() {
            let child = self.children.pop().unwrap();
            let child_ptr = child.as_internal_mut().unwrap();
            std::mem::swap(&mut *child_ptr, self);
            self.parent = None;
            let ptr = self.into();
            // TODO: extract reset parent?
            for child in self.children.iter_mut() {
                child.set_parent(ptr);
            }

            child_ptr.parent = None;
            child_ptr.children.clear();
            ans.insert(&*child_ptr as *const _);
        }

        ans
    }

    #[inline]
    fn is_root(&self) -> bool {
        self.parent.is_none()
    }

    fn _insert_with_split(
        &mut self,
        child_index: usize,
        new: &'a mut Node<'a, T, A>,
    ) -> Result<(), &'a mut Node<'a, T, A>> {
        if self.children.len() == A::MAX_CHILDREN_NUM {
            let ans = self._split();
            if child_index < self.children.len() {
                new.set_parent(self.into());
                self.children.insert(child_index, new);
            } else {
                new.set_parent((&mut *ans.as_internal_mut().unwrap()).into());
                ans.as_internal_mut()
                    .unwrap()
                    .children
                    .insert(child_index - self.children.len(), new);
            }

            Err(ans)
        } else {
            new.set_parent(self.into());
            self.children.insert(child_index, new);
            Ok(())
        }
    }
}

impl<'a, T: Rle, A: RleTreeTrait<T>> Debug for InternalNode<'a, T, A> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        let mut debug_struct = f.debug_struct("InternalNode");
        debug_struct.field("children", &self.children);
        debug_struct.field("cache", &self.cache);
        debug_struct.field("children_num", &self.children.len());
        debug_struct.finish()
    }
}