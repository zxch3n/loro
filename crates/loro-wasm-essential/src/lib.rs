//! Loro WASM bindings.
#![allow(non_snake_case)]
#![allow(clippy::empty_docs)]
#![allow(clippy::doc_lazy_continuation)]
// #![warn(missing_docs)]

mod convert;

use convert::{import_status_to_js_value, js_to_version_vector};
use js_sys::{Array, Object, Reflect, Uint8Array};
use loro_internal::{
    change::Lamport,
    encoding::ImportBlobMetadata,
    id::{Counter, PeerID, ID},
    json::JsonSchema,
    loro::ExportMode,
    version::Frontiers,
    FxHashMap, LoroDoc as LoroDocInner, LoroResult, VersionVector as InternalVersionVector,
};
use rle::HasLength;
use serde::Serialize;
use std::{cmp::Ordering, sync::Arc};
use wasm_bindgen::{__rt::IntoJsResult, prelude::*};
mod log;

#[wasm_bindgen]
pub fn encodeFrontiers(frontiers: Vec<JsID>) -> JsResult<Vec<u8>> {
    let frontiers = ids_to_frontiers(frontiers)?;
    let encoded = frontiers.encode();
    Ok(encoded)
}

#[wasm_bindgen]
pub fn decodeFrontiers(bytes: &[u8]) -> JsResult<JsIDs> {
    let frontiers =
        Frontiers::decode(bytes).map_err(|_| JsError::new("Invalid frontiers binary data"))?;
    Ok(frontiers_to_ids(&frontiers))
}

type JsResult<T> = Result<T, JsValue>;

/// The CRDTs document. Loro supports different CRDTs include [**List**](LoroList),
/// [**RichText**](LoroText), [**Map**](LoroMap) and [**Movable Tree**](LoroTree),
/// you could build all kind of applications by these.
///
/// @example
/// ```ts
/// import { LoroDoc } from "loro-crdt"
///
/// const loro = new LoroDoc();
/// const text = loro.getText("text");
/// const list = loro.getList("list");
/// const map = loro.getMap("Map");
/// const tree = loro.getTree("tree");
/// ```
#[wasm_bindgen]
pub struct LoroDoc(Arc<LoroDocInner>);

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "number | bigint | `${number}`")]
    pub type JsIntoPeerID;
    #[wasm_bindgen(typescript_type = "PeerID")]
    pub type JsStrPeerID;
    #[wasm_bindgen(typescript_type = "ContainerID")]
    pub type JsContainerID;
    #[wasm_bindgen(typescript_type = "ContainerID | string")]
    pub type JsIntoContainerID;
    #[wasm_bindgen(typescript_type = "Transaction | LoroDoc")]
    pub type JsTransaction;
    #[wasm_bindgen(typescript_type = "string | undefined")]
    pub type JsOrigin;
    #[wasm_bindgen(typescript_type = "{ peer: PeerID, counter: number }")]
    pub type JsID;
    #[wasm_bindgen(typescript_type = "{ peer: PeerID, counter: number }[]")]
    pub type JsIDs;
    #[wasm_bindgen(typescript_type = "{ start: number, end: number }")]
    pub type JsRange;
    #[wasm_bindgen(typescript_type = "number|bool|string|null")]
    pub type JsMarkValue;
    #[wasm_bindgen(typescript_type = "TreeID")]
    pub type JsTreeID;
    #[wasm_bindgen(typescript_type = "TreeID | undefined")]
    pub type JsParentTreeID;
    #[wasm_bindgen(typescript_type = "{ withDeleted: boolean }")]
    pub type JsGetNodesProp;
    #[wasm_bindgen(typescript_type = "LoroTreeNode | undefined")]
    pub type JsTreeNodeOrUndefined;
    #[wasm_bindgen(typescript_type = "string | undefined")]
    pub type JsPositionOrUndefined;
    #[wasm_bindgen(typescript_type = "Delta<string>[]")]
    pub type JsStringDelta;
    #[wasm_bindgen(typescript_type = "Map<PeerID, number>")]
    pub type JsVersionVectorMap;
    #[wasm_bindgen(typescript_type = "Map<PeerID, Change[]>")]
    pub type JsChanges;
    #[wasm_bindgen(typescript_type = "Change")]
    pub type JsChange;
    #[wasm_bindgen(typescript_type = "Change | undefined")]
    pub type JsChangeOrUndefined;
    #[wasm_bindgen(
        typescript_type = "Map<PeerID, number> | Uint8Array | VersionVector | undefined | null"
    )]
    pub type JsIntoVersionVector;
    #[wasm_bindgen(typescript_type = "Container")]
    pub type JsContainer;
    #[wasm_bindgen(typescript_type = "Value")]
    pub type JsLoroValue;
    #[wasm_bindgen(typescript_type = "Value | Container")]
    pub type JsValueOrContainer;
    #[wasm_bindgen(typescript_type = "Value | Container | undefined")]
    pub type JsValueOrContainerOrUndefined;
    #[wasm_bindgen(typescript_type = "Container | undefined")]
    pub type JsContainerOrUndefined;
    #[wasm_bindgen(typescript_type = "LoroText | undefined")]
    pub type JsLoroTextOrUndefined;
    #[wasm_bindgen(typescript_type = "LoroMap | undefined")]
    pub type JsLoroMapOrUndefined;
    #[wasm_bindgen(typescript_type = "LoroList | undefined")]
    pub type JsLoroListOrUndefined;
    #[wasm_bindgen(typescript_type = "LoroTree | undefined")]
    pub type JsLoroTreeOrUndefined;
    #[wasm_bindgen(typescript_type = "[string, Value | Container]")]
    pub type MapEntry;
    #[wasm_bindgen(typescript_type = "{[key: string]: { expand: 'before'|'after'|'none'|'both' }}")]
    pub type JsTextStyles;
    #[wasm_bindgen(typescript_type = "Delta<string>[]")]
    pub type JsDelta;
    #[wasm_bindgen(typescript_type = "-1 | 1 | 0 | undefined")]
    pub type JsPartialOrd;
    #[wasm_bindgen(typescript_type = "'Tree'|'Map'|'List'|'Text'")]
    pub type JsContainerKind;
    #[wasm_bindgen(typescript_type = "'Text'")]
    pub type JsTextStr;
    #[wasm_bindgen(typescript_type = "'Tree'")]
    pub type JsTreeStr;
    #[wasm_bindgen(typescript_type = "'Map'")]
    pub type JsMapStr;
    #[wasm_bindgen(typescript_type = "'List'")]
    pub type JsListStr;
    #[wasm_bindgen(typescript_type = "'MovableList'")]
    pub type JsMovableListStr;
    #[wasm_bindgen(typescript_type = "ImportBlobMetadata")]
    pub type JsImportBlobMetadata;
    #[wasm_bindgen(typescript_type = "Side")]
    pub type JsSide;
    #[wasm_bindgen(typescript_type = "{ update?: Cursor, offset: number, side: Side }")]
    pub type JsCursorQueryAns;
    #[wasm_bindgen(typescript_type = "UndoConfig")]
    pub type JsUndoConfig;
    #[wasm_bindgen(typescript_type = "JsonSchema")]
    pub type JsJsonSchema;
    #[wasm_bindgen(typescript_type = "string | JsonSchema")]
    pub type JsJsonSchemaOrString;
    #[wasm_bindgen(typescript_type = "ExportMode")]
    pub type JsExportMode;
    #[wasm_bindgen(typescript_type = "{ origin?: string, timestamp?: number, message?: string }")]
    pub type JsCommitOption;
    #[wasm_bindgen(typescript_type = "ImportStatus")]
    pub type JsImportStatus;
    #[wasm_bindgen(typescript_type = "(change: ChangeMeta) => boolean")]
    pub type JsTravelChangeFunction;
    #[wasm_bindgen(typescript_type = "(string|number)[]")]
    pub type JsContainerPath;
}

fn ids_to_frontiers(ids: Vec<JsID>) -> JsResult<Frontiers> {
    let mut frontiers = Frontiers::default();
    for id in ids {
        let id = js_id_to_id(id)?;
        frontiers.push(id);
    }

    Ok(frontiers)
}

fn id_to_js(id: &ID) -> JsValue {
    let obj = Object::new();
    Reflect::set(&obj, &"peer".into(), &id.peer.to_string().into()).unwrap();
    Reflect::set(&obj, &"counter".into(), &id.counter.into()).unwrap();
    let value: JsValue = obj.into_js_result().unwrap();
    value
}

fn peer_id_to_js(peer: PeerID) -> JsStrPeerID {
    let v: JsValue = peer.to_string().into();
    v.into()
}

fn js_id_to_id(id: JsID) -> Result<ID, JsValue> {
    let peer = js_peer_to_peer(Reflect::get(&id, &"peer".into())?)?;
    let counter = Reflect::get(&id, &"counter".into())?.as_f64().unwrap() as Counter;
    let id = ID::new(peer, counter);
    Ok(id)
}

fn frontiers_to_ids(frontiers: &Frontiers) -> JsIDs {
    let js_arr = Array::new();
    for id in frontiers.iter() {
        let value = id_to_js(&id);
        js_arr.push(&value);
    }

    let value: JsValue = js_arr.into();
    value.into()
}

#[derive(Debug, Clone, Serialize)]
struct StringID {
    peer: String,
    counter: Counter,
}

#[derive(Debug, Clone, Serialize)]
struct ChangeMeta {
    lamport: Lamport,
    length: u32,
    peer: String,
    counter: Counter,
    deps: Vec<StringID>,
    timestamp: f64,
    message: Option<Arc<str>>,
}

impl ChangeMeta {
    fn to_js(&self) -> JsValue {
        let s = serde_wasm_bindgen::Serializer::new();
        self.serialize(&s).unwrap()
    }
}

#[wasm_bindgen]
impl LoroDoc {
    /// Create a new loro document.
    ///
    /// New document will have a random peer id.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        let doc = LoroDocInner::new();
        doc.start_auto_commit();
        Self(Arc::new(doc))
    }

    /// Get the version vector of the current document state.
    ///
    /// If you checkout to a specific version, the version vector will change.
    #[inline(always)]
    pub fn version(&self) -> VersionVector {
        VersionVector(self.0.state_vv())
    }

    /// The doc only contains the history since this version
    ///
    /// This is empty if the doc is not shallow.
    ///
    /// The ops included by the shallow history start version vector are not in the doc.
    #[wasm_bindgen(js_name = "shallowSinceVV")]
    pub fn shallow_since_vv(&self) -> VersionVector {
        VersionVector(InternalVersionVector::from_im_vv(
            &self.0.shallow_since_vv(),
        ))
    }

    /// Check if the doc contains the full history.
    #[wasm_bindgen(js_name = "isShallow")]
    pub fn is_shallow(&self) -> bool {
        self.0.is_shallow()
    }

    /// The doc only contains the history since this version
    ///
    /// This is empty if the doc is not shallow.
    ///
    /// The ops included by the shallow history start frontiers are not in the doc.
    #[wasm_bindgen(js_name = "shallowSinceFrontiers")]
    pub fn shallow_since_frontiers(&self) -> JsIDs {
        frontiers_to_ids(&self.0.shallow_since_frontiers())
    }

    /// Get the version vector of the latest known version in OpLog.
    ///
    /// If you checkout to a specific version, this version vector will not change.
    #[wasm_bindgen(js_name = "oplogVersion")]
    pub fn oplog_version(&self) -> VersionVector {
        VersionVector(self.0.oplog_vv())
    }

    /// Get the [frontiers](https://loro.dev/docs/advanced/version_deep_dive) of the current document state.
    ///
    /// If you checkout to a specific version, this value will change.
    #[inline]
    pub fn frontiers(&self) -> JsIDs {
        frontiers_to_ids(&self.0.state_frontiers())
    }

    /// Get the [frontiers](https://loro.dev/docs/advanced/version_deep_dive) of the latest version in OpLog.
    ///
    /// If you checkout to a specific version, this value will not change.
    #[inline(always)]
    #[wasm_bindgen(js_name = "oplogFrontiers")]
    pub fn oplog_frontiers(&self) -> JsIDs {
        frontiers_to_ids(&self.0.oplog_frontiers())
    }

    /// Compare the version of the OpLog with the specified frontiers.
    ///
    /// This method is useful to compare the version by only a small amount of data.
    ///
    /// This method returns an integer indicating the relationship between the version of the OpLog (referred to as 'self')
    /// and the provided 'frontiers' parameter:
    ///
    /// - -1: The version of 'self' is either less than 'frontiers' or is non-comparable (parallel) to 'frontiers',
    ///        indicating that it is not definitively less than 'frontiers'.
    /// - 0: The version of 'self' is equal to 'frontiers'.
    /// - 1: The version of 'self' is greater than 'frontiers'.
    ///
    /// # Internal
    ///
    /// Frontiers cannot be compared without the history of the OpLog.
    ///
    #[wasm_bindgen(js_name = "cmpWithFrontiers")]
    pub fn cmp_with_frontiers(&self, frontiers: Vec<JsID>) -> JsResult<i32> {
        let frontiers = ids_to_frontiers(frontiers)?;
        Ok(match self.0.cmp_with_frontiers(&frontiers) {
            Ordering::Less => -1,
            Ordering::Greater => 1,
            Ordering::Equal => 0,
        })
    }

    /// Compare the ordering of two Frontiers.
    ///
    /// It's assumed that both Frontiers are included by the doc. Otherwise, an error will be thrown.
    ///
    /// Return value:
    ///
    /// - -1: a < b
    /// - 0: a == b
    /// - 1: a > b
    /// - undefined: a ∥ b: a and b are concurrent
    #[wasm_bindgen(js_name = "cmpFrontiers")]
    pub fn cmp_frontiers(&self, a: Vec<JsID>, b: Vec<JsID>) -> JsResult<JsPartialOrd> {
        let a = ids_to_frontiers(a)?;
        let b = ids_to_frontiers(b)?;
        let c = self
            .0
            .cmp_frontiers(&a, &b)
            .map_err(|e| JsError::new(&e.to_string()))?;
        if let Some(c) = c {
            let v: JsValue = match c {
                Ordering::Less => -1,
                Ordering::Greater => 1,
                Ordering::Equal => 0,
            }
            .into();
            Ok(v.into())
        } else {
            Ok(JsValue::UNDEFINED.into())
        }
    }
    /// Export the document based on the specified ExportMode.
    ///
    /// @param mode - The export mode to use. Can be one of:
    ///   - `{ mode: "snapshot" }`: Export a full snapshot of the document.
    ///   - `{ mode: "update", from?: VersionVector }`: Export updates from the given version vector.
    ///   - `{ mode: "updates-in-range", spans: { id: ID, len: number }[] }`: Export updates within the specified ID spans.
    ///   - `{ mode: "shallow-snapshot", frontiers: Frontiers }`: Export a garbage-collected snapshot up to the given frontiers.
    ///
    /// @returns A byte array containing the exported data.
    ///
    /// @example
    /// ```ts
    /// import { LoroDoc, LoroText } from "loro-crdt";
    ///
    /// const doc = new LoroDoc();
    /// doc.setPeerId("1");
    /// doc.getText("text").update("Hello World");
    ///
    /// // Export a full snapshot
    /// const snapshotBytes = doc.export({ mode: "snapshot" });
    ///
    /// // Export updates from a specific version
    /// const vv = doc.oplogVersion();
    /// doc.getText("text").update("Hello Loro");
    /// const updateBytes = doc.export({ mode: "update", from: vv });
    ///
    /// // Export a shallow snapshot that only includes the history since the frontiers
    /// const shallowBytes = doc.export({ mode: "shallow-snapshot", frontiers: doc.oplogFrontiers() });
    ///
    /// // Export updates within specific ID spans
    /// const spanBytes = doc.export({
    ///   mode: "updates-in-range",
    ///   spans: [{ id: { peer: "1", counter: 0 }, len: 10 }]
    /// });
    /// ```
    pub fn export(&self, mode: JsExportMode) -> JsResult<Vec<u8>> {
        let export_mode = js_to_export_mode(mode)
            .map_err(|e| JsValue::from_str(&format!("Invalid export mode. Error: {:?}", e)))?;
        Ok(self.0.export(export_mode)?)
    }

    /// Export updates in the given range in JSON format.
    #[wasm_bindgen(js_name = "exportJsonUpdates")]
    pub fn export_json_updates(
        &self,
        start_vv: Option<VersionVector>,
        end_vv: Option<VersionVector>,
    ) -> JsResult<JsJsonSchema> {
        let mut json_start_vv = Default::default();
        if let Some(vv) = start_vv {
            json_start_vv = vv.0;
        }
        let mut json_end_vv = self.oplog_version().0;
        if let Some(vv) = end_vv {
            json_end_vv = vv.0;
        }
        let json_schema = self.0.export_json_updates(&json_start_vv, &json_end_vv);
        let s = serde_wasm_bindgen::Serializer::new().serialize_maps_as_objects(true);
        let v = json_schema
            .serialize(&s)
            .map_err(std::convert::Into::<JsValue>::into)?;
        Ok(v.into())
    }

    /// Import updates from the JSON format.
    ///
    /// only supports backward compatibility but not forward compatibility.
    #[wasm_bindgen(js_name = "importJsonUpdates")]
    pub fn import_json_updates(&self, json: JsJsonSchemaOrString) -> JsResult<JsImportStatus> {
        let json: JsValue = json.into();
        if JsValue::is_string(&json) {
            let json_str = json.as_string().unwrap();
            let status = self
                .0
                .import_json_updates(json_str.as_str())
                .map_err(JsValue::from)?;
            return Ok(import_status_to_js_value(status).into());
        }
        let json_schema: JsonSchema = serde_wasm_bindgen::from_value(json)?;
        let status = self.0.import_json_updates(json_schema)?;
        Ok(import_status_to_js_value(status).into())
    }

    /// Import snapshot or updates into current doc.
    ///
    /// Note:
    /// - Updates within the current version will be ignored
    /// - Updates with missing dependencies will be pending until the dependencies are received
    ///
    /// @example
    /// ```ts
    /// import { LoroDoc } from "loro-crdt";
    ///
    /// const doc = new LoroDoc();
    /// const text = doc.getText("text");
    /// text.insert(0, "Hello");
    /// // get all updates of the doc
    /// const updates = doc.export({ mode: "update" });
    /// const snapshot = doc.export({ mode: "snapshot" });
    /// const doc2 = new LoroDoc();
    /// // import snapshot
    /// doc2.import(snapshot);
    /// // or import updates
    /// doc2.import(updates);
    /// ```
    pub fn import(&self, update_or_snapshot: &[u8]) -> JsResult<JsImportStatus> {
        let status = self.0.import(update_or_snapshot)?;
        Ok(import_status_to_js_value(status).into())
    }

    /// Import a batch of updates.
    ///
    /// It's more efficient than importing updates one by one.
    ///
    /// @example
    /// ```ts
    /// import { LoroDoc } from "loro-crdt";
    ///
    /// const doc = new LoroDoc();
    /// const text = doc.getText("text");
    /// text.insert(0, "Hello");
    /// const updates = doc.export({ mode: "update" });
    /// const snapshot = doc.export({ mode: "snapshot" });
    /// const doc2 = new LoroDoc();
    /// doc2.importUpdateBatch([snapshot, updates]);
    /// ```
    #[wasm_bindgen(js_name = "importUpdateBatch")]
    pub fn import_update_batch(&mut self, data: Array) -> JsResult<()> {
        let data = data
            .iter()
            .map(|x| {
                let arr: Uint8Array = Uint8Array::new(&x);
                arr.to_vec()
            })
            .collect::<Vec<_>>();
        if data.is_empty() {
            return Ok(());
        }
        Ok(self.0.import_batch(&data)?)
    }

    /// Get the shallow json format of the document state.
    ///
    /// @example
    /// ```ts
    /// import { LoroDoc } from "loro-crdt";
    ///
    /// const doc = new LoroDoc();
    /// const list = doc.getList("list");
    /// const tree = doc.getTree("tree");
    /// const map = doc.getMap("map");
    /// const shallowValue = doc.getShallowValue();
    /// /*
    /// {"list": ..., "tree": ..., "map": ...}
    ///  *\/
    /// console.log(shallowValue);
    /// ```
    #[wasm_bindgen(js_name = "getShallowValue")]
    pub fn get_shallow_value(&self) -> JsResult<JsValue> {
        let json = self.0.get_value();
        Ok(json.into())
    }

    /// Get the json format of the entire document state.
    ///
    /// @example
    /// ```ts
    /// import { LoroDoc, LoroText, LoroMap } from "loro-crdt";
    ///
    /// const doc = new LoroDoc();
    /// const list = doc.getList("list");
    /// list.insert(0, "Hello");
    /// const text = list.insertContainer(0, new LoroText());
    /// text.insert(0, "Hello");
    /// const map = list.insertContainer(1, new LoroMap());
    /// map.set("foo", "bar");
    /// /*
    /// {"list": ["Hello", {"foo": "bar"}]}
    ///  *\/
    /// console.log(doc.toJSON());
    /// ```
    #[wasm_bindgen(js_name = "toJSON")]
    pub fn to_json(&self) -> JsResult<JsValue> {
        let json = self.0.get_deep_value();
        Ok(json.into())
    }

    /// Debug the size of the history
    #[wasm_bindgen(js_name = "debugHistory")]
    pub fn debug_history(&self) {
        let borrow_mut = &self.0;
        let oplog = borrow_mut.oplog().try_lock().unwrap();
        console_log!("{:#?}", oplog.diagnose_size());
    }

    /// Get the number of changes in the oplog.
    pub fn changeCount(&self) -> usize {
        let borrow_mut = &self.0;
        let oplog = borrow_mut.oplog().try_lock().unwrap();
        oplog.len_changes()
    }

    /// Get the number of ops in the oplog.
    pub fn opCount(&self) -> usize {
        self.0.len_ops()
    }

    /// Get all of changes in the oplog.
    ///
    /// Note: this method is expensive when the oplog is large. O(n)
    ///
    /// @example
    /// ```ts
    /// import { LoroDoc, LoroText } from "loro-crdt";
    ///
    /// const doc = new LoroDoc();
    /// const text = doc.getText("text");
    /// text.insert(0, "Hello");
    /// const changes = doc.getAllChanges();
    ///
    /// for (let [peer, c] of changes.entries()){
    ///     console.log("peer: ", peer);
    ///     for (let change of c){
    ///         console.log("change: ", change);
    ///     }
    /// }
    /// ```
    #[wasm_bindgen(js_name = "getAllChanges")]
    pub fn get_all_changes(&self) -> JsChanges {
        let borrow_mut = &self.0;
        let oplog = borrow_mut.oplog().try_lock().unwrap();
        let mut changes: FxHashMap<PeerID, Vec<ChangeMeta>> = FxHashMap::default();
        oplog.change_store().visit_all_changes(&mut |c| {
            let change_meta = ChangeMeta {
                lamport: c.lamport(),
                length: c.atom_len() as u32,
                peer: c.peer().to_string(),
                counter: c.id().counter,
                deps: c
                    .deps()
                    .iter()
                    .map(|dep| StringID {
                        peer: dep.peer.to_string(),
                        counter: dep.counter,
                    })
                    .collect(),
                timestamp: c.timestamp() as f64,
                message: c.message().cloned(),
            };
            changes.entry(c.peer()).or_default().push(change_meta);
        });

        let ans = js_sys::Map::new();
        for (peer_id, changes) in changes {
            let row = js_sys::Array::new_with_length(changes.len() as u32);
            for (i, change) in changes.iter().enumerate() {
                row.set(i as u32, change.to_js());
            }
            ans.set(&peer_id.to_string().into(), &row);
        }

        let value: JsValue = ans.into();
        value.into()
    }

    /// Get the change that contains the specific ID
    #[wasm_bindgen(js_name = "getChangeAt")]
    pub fn get_change_at(&self, id: JsID) -> JsResult<JsChange> {
        let id = js_id_to_id(id)?;
        let borrow_mut = &self.0;
        let oplog = borrow_mut.oplog().try_lock().unwrap();
        let change = oplog
            .get_change_at(id)
            .ok_or_else(|| JsError::new(&format!("Change {:?} not found", id)))?;
        let change = ChangeMeta {
            lamport: change.lamport(),
            length: change.atom_len() as u32,
            peer: change.peer().to_string(),
            counter: change.id().counter,
            deps: change
                .deps()
                .iter()
                .map(|dep| StringID {
                    peer: dep.peer.to_string(),
                    counter: dep.counter,
                })
                .collect(),
            timestamp: change.timestamp() as f64,
            message: change.message().cloned(),
        };
        Ok(change.to_js().into())
    }

    /// Get the change of with specific peer_id and lamport <= given lamport
    #[wasm_bindgen(js_name = "getChangeAtLamport")]
    pub fn get_change_at_lamport(
        &self,
        peer_id: &str,
        lamport: u32,
    ) -> JsResult<JsChangeOrUndefined> {
        let borrow_mut = &self.0;
        let oplog = borrow_mut.oplog().try_lock().unwrap();
        let Some(change) =
            oplog.get_change_with_lamport_lte(peer_id.parse().unwrap_throw(), lamport)
        else {
            return Ok(JsValue::UNDEFINED.into());
        };

        let change = ChangeMeta {
            lamport: change.lamport(),
            length: change.atom_len() as u32,
            peer: change.peer().to_string(),
            counter: change.id().counter,
            deps: change
                .deps()
                .iter()
                .map(|dep| StringID {
                    peer: dep.peer.to_string(),
                    counter: dep.counter,
                })
                .collect(),
            timestamp: change.timestamp() as f64,
            message: change.message().cloned(),
        };
        Ok(change.to_js().into())
    }

    /// Get all ops of the change that contains the specific ID
    #[wasm_bindgen(js_name = "getOpsInChange")]
    pub fn get_ops_in_change(&self, id: JsID) -> JsResult<Vec<JsValue>> {
        let id = js_id_to_id(id)?;
        let borrow_mut = &self.0;
        let oplog = borrow_mut.oplog().try_lock().unwrap();
        let change = oplog
            .get_remote_change_at(id)
            .ok_or_else(|| JsError::new(&format!("Change {:?} not found", id)))?;
        let ops = change
            .ops()
            .iter()
            .map(|op| serde_wasm_bindgen::to_value(op).unwrap())
            .collect::<Vec<_>>();
        Ok(ops)
    }

    /// Convert frontiers to a version vector
    ///
    /// Learn more about frontiers and version vector [here](https://loro.dev/docs/advanced/version_deep_dive)
    ///
    /// @example
    /// ```ts
    /// import { LoroDoc } from "loro-crdt";
    ///
    /// const doc = new LoroDoc();
    /// const text = doc.getText("text");
    /// text.insert(0, "Hello");
    /// const frontiers = doc.frontiers();
    /// const version = doc.frontiersToVV(frontiers);
    /// ```
    #[wasm_bindgen(js_name = "frontiersToVV")]
    pub fn frontiers_to_vv(&self, frontiers: Vec<JsID>) -> JsResult<VersionVector> {
        let frontiers = ids_to_frontiers(frontiers)?;
        let borrow_mut = &self.0;
        let oplog = borrow_mut.oplog().try_lock().unwrap();
        oplog
            .dag()
            .frontiers_to_vv(&frontiers)
            .map(VersionVector)
            .ok_or_else(|| JsError::new("Frontiers not found").into())
    }

    /// Convert a version vector to frontiers
    ///
    /// @example
    /// ```ts
    /// import { LoroDoc } from "loro-crdt";
    ///
    /// const doc = new LoroDoc();
    /// const text = doc.getText("text");
    /// text.insert(0, "Hello");
    /// const version = doc.version();
    /// const frontiers = doc.vvToFrontiers(version);
    /// ```
    #[wasm_bindgen(js_name = "vvToFrontiers")]
    pub fn vv_to_frontiers(&self, vv: &VersionVector) -> JsResult<JsIDs> {
        let f = self
            .0
            .oplog()
            .try_lock()
            .unwrap()
            .dag()
            .vv_to_frontiers(&vv.0);
        Ok(frontiers_to_ids(&f))
    }
}

/// [VersionVector](https://en.wikipedia.org/wiki/Version_vector)
/// is a map from [PeerID] to [Counter]. Its a right-open interval.
///
/// i.e. a [VersionVector] of `{A: 1, B: 2}` means that A has 1 atomic op and B has 2 atomic ops,
/// thus ID of `{client: A, counter: 1}` is out of the range.
#[wasm_bindgen]
#[derive(Debug, Default)]
pub struct VersionVector(pub(crate) InternalVersionVector);

#[wasm_bindgen]
impl VersionVector {
    /// Create a new version vector.
    #[wasm_bindgen(constructor)]
    pub fn new(value: JsIntoVersionVector) -> JsResult<VersionVector> {
        let value: JsValue = value.into();
        if value.is_null() || value.is_undefined() {
            return Ok(Self::default());
        }

        let is_bytes = value.is_instance_of::<js_sys::Uint8Array>();
        if is_bytes {
            let bytes = js_sys::Uint8Array::from(value.clone());
            let bytes = bytes.to_vec();
            return VersionVector::decode(&bytes);
        }

        VersionVector::from_json(JsVersionVectorMap::from(value))
    }

    /// Create a new version vector from a Map.
    #[wasm_bindgen(js_name = "parseJSON", method)]
    pub fn from_json(version: JsVersionVectorMap) -> JsResult<VersionVector> {
        let map: JsValue = version.into();
        let map: js_sys::Map = map.into();
        let mut vv = InternalVersionVector::new();
        for pair in map.entries() {
            let pair = pair.unwrap_throw();
            let key = Reflect::get(&pair, &0.into()).unwrap_throw();
            let peer_id = key.as_string().expect_throw("PeerID must be string");
            let value = Reflect::get(&pair, &1.into()).unwrap_throw();
            let counter = value.as_f64().expect_throw("Invalid counter") as Counter;
            vv.insert(
                peer_id
                    .parse()
                    .expect_throw(&format!("{} cannot be parsed as u64", peer_id)),
                counter,
            );
        }

        Ok(Self(vv))
    }

    /// Convert the version vector to a Map
    #[wasm_bindgen(js_name = "toJSON", method)]
    pub fn to_json(&self) -> JsVersionVectorMap {
        let vv = &self.0;
        let map = js_sys::Map::new();
        for (k, v) in vv.iter() {
            let k = k.to_string().into();
            let v = JsValue::from(*v);
            map.set(&k, &v);
        }

        let value: JsValue = map.into();
        JsVersionVectorMap::from(value)
    }

    /// Encode the version vector into a Uint8Array.
    pub fn encode(&self) -> Vec<u8> {
        self.0.encode()
    }

    /// Decode the version vector from a Uint8Array.
    pub fn decode(bytes: &[u8]) -> JsResult<VersionVector> {
        let vv = InternalVersionVector::decode(bytes)?;
        Ok(Self(vv))
    }

    /// Get the counter of a peer.
    pub fn get(&self, peer_id: JsIntoPeerID) -> JsResult<Option<Counter>> {
        let id = js_peer_to_peer(peer_id.into())?;
        Ok(self.0.get(&id).copied())
    }

    /// Compare the version vector with another version vector.
    ///
    /// If they are concurrent, return undefined.
    pub fn compare(&self, other: &VersionVector) -> Option<i32> {
        self.0.partial_cmp(&other.0).map(|o| match o {
            std::cmp::Ordering::Less => -1,
            std::cmp::Ordering::Equal => 0,
            std::cmp::Ordering::Greater => 1,
        })
    }

    /// set the exclusive ending point. target id will NOT be included by self
    pub fn setEnd(&mut self, id: JsID) -> JsResult<()> {
        let id = js_id_to_id(id)?;
        self.0.set_end(id);
        Ok(())
    }

    /// set the inclusive ending point. target id will be included
    pub fn setLast(&mut self, id: JsID) -> JsResult<()> {
        let id = js_id_to_id(id)?;
        self.0.set_last(id);
        Ok(())
    }

    pub fn remove(&mut self, peer: JsStrPeerID) -> LoroResult<()> {
        let peer = js_peer_to_peer(peer.into())?;
        self.0.remove(&peer);
        Ok(())
    }

    pub fn length(&self) -> usize {
        self.0.len()
    }
}

const ID_CONVERT_ERROR: &str = "Invalid peer id. It must be a number, a BigInt, or a decimal string that can be parsed to a unsigned 64-bit integer";
fn js_peer_to_peer(value: JsValue) -> JsResult<u64> {
    if value.is_bigint() {
        let bigint = js_sys::BigInt::from(value);
        let v: u64 = bigint
            .try_into()
            .map_err(|_| JsValue::from_str(ID_CONVERT_ERROR))?;
        Ok(v)
    } else if value.is_string() {
        let v: u64 = value
            .as_string()
            .unwrap()
            .parse()
            .expect_throw(ID_CONVERT_ERROR);
        Ok(v)
    } else if let Some(v) = value.as_f64() {
        Ok(v as u64)
    } else {
        Err(JsValue::from_str(ID_CONVERT_ERROR))
    }
}

/// Decode the metadata of the import blob.
///
/// This method is useful to get the following metadata of the import blob:
///
/// - startVersionVector
/// - endVersionVector
/// - startTimestamp
/// - endTimestamp
/// - mode
/// - changeNum
#[wasm_bindgen(js_name = "decodeImportBlobMeta")]
pub fn decode_import_blob_meta(
    blob: &[u8],
    check_checksum: bool,
) -> JsResult<JsImportBlobMetadata> {
    let meta: ImportBlobMetadata = LoroDocInner::decode_import_blob_meta(blob, check_checksum)?;
    Ok(meta.into())
}

fn js_to_export_mode(js_mode: JsExportMode) -> JsResult<ExportMode<'static>> {
    let js_value: JsValue = js_mode.into();
    let mode = js_sys::Reflect::get(&js_value, &JsValue::from_str("mode"))?
        .as_string()
        .ok_or_else(|| JsError::new("Invalid mode"))?;

    match mode.as_str() {
        "update" => {
            let from = js_sys::Reflect::get(&js_value, &JsValue::from_str("from"))?;
            if from.is_undefined() {
                Ok(ExportMode::all_updates())
            } else {
                let from = js_to_version_vector(from)?;
                // TODO: PERF: avoid this clone
                Ok(ExportMode::updates_owned(from.0.clone()))
            }
        }
        "snapshot" => Ok(ExportMode::Snapshot),
        "shallow-snapshot" => {
            let frontiers: JsValue =
                js_sys::Reflect::get(&js_value, &JsValue::from_str("frontiers"))?;
            let frontiers: Vec<JsID> = js_sys::try_iter(&frontiers)?
                .ok_or_else(|| JsError::new("frontiers is not iterable"))?
                .map(|res| res.map(JsID::from))
                .collect::<Result<_, _>>()?;
            let frontiers = ids_to_frontiers(frontiers)?;
            Ok(ExportMode::shallow_snapshot_owned(frontiers))
        }
        "updates-in-range" => {
            let spans = js_sys::Reflect::get(&js_value, &JsValue::from_str("spans"))?;
            let spans: js_sys::Array = spans.dyn_into()?;
            let mut rust_spans = Vec::new();
            for span in spans.iter() {
                let id = js_sys::Reflect::get(&span, &JsValue::from_str("id"))?;
                let len = js_sys::Reflect::get(&span, &JsValue::from_str("len"))?
                    .as_f64()
                    .ok_or_else(|| JsError::new("Invalid len"))?;
                let id = js_id_to_id(id.into())?;
                rust_spans.push(id.to_span(len as usize));
            }
            Ok(ExportMode::updates_in_range(rust_spans))
        }
        _ => Err(JsError::new("Invalid export mode").into()),
    }
}

#[wasm_bindgen(typescript_custom_section)]
const TYPES: &'static str = r#"
/**
* Container types supported by loro.
*
* It is most commonly used to specify the type of sub-container to be created.
* @example
* ```ts
* import { LoroDoc, LoroText } from "loro-crdt";
*
* const doc = new LoroDoc();
* const list = doc.getList("list");
* list.insert(0, 100);
* const text = list.insertContainer(1, new LoroText());
* ```
*/
export type ContainerType = "Text" | "Map" | "List"| "Tree" | "MovableList";

export type PeerID = `${number}`;
/**
* The unique id of each container.
*
* @example
* ```ts
* import { LoroDoc } from "loro-crdt";
*
* const doc = new LoroDoc();
* const list = doc.getList("list");
* const containerId = list.id;
* ```
*/
export type ContainerID =
  | `cid:root-${string}:${ContainerType}`
  | `cid:${number}@${PeerID}:${ContainerType}`;

/**
 * The unique id of each tree node.
 */
export type TreeID = `${number}@${PeerID}`;

interface LoroDoc {
    /**
     * Export updates from the specific version to the current version
     *
     * @deprecated Use `export({mode: "update", from: version})` instead
     *
     *  @example
     *  ```ts
     *  import { LoroDoc } from "loro-crdt";
     *
     *  const doc = new LoroDoc();
     *  const text = doc.getText("text");
     *  text.insert(0, "Hello");
     *  // get all updates of the doc
     *  const updates = doc.exportFrom();
     *  const version = doc.oplogVersion();
     *  text.insert(5, " World");
     *  // get updates from specific version to the latest version
     *  const updates2 = doc.exportFrom(version);
     *  ```
     */
    exportFrom(version?: VersionVector): Uint8Array;
    /**
     *
     *  Get the container corresponding to the container id
     *
     *
     *  @example
     *  ```ts
     *  import { LoroDoc } from "loro-crdt";
     *
     *  const doc = new LoroDoc();
     *  let text = doc.getText("text");
     *  const textId = text.id;
     *  text = doc.getContainerById(textId);
     *  ```
     */
    getContainerById(id: ContainerID): Container;

    /**
     * Subscribe to updates from local edits.
     *
     * This method allows you to listen for local changes made to the document.
     * It's useful for syncing changes with other instances or saving updates.
     *
     * @param f - A callback function that receives a Uint8Array containing the update data.
     * @returns A function to unsubscribe from the updates.
     *
     * @example
     * ```ts
     * const loro = new Loro();
     * const text = loro.getText("text");
     *
     * const unsubscribe = loro.subscribeLocalUpdates((update) => {
     *   console.log("Local update received:", update);
     *   // You can send this update to other Loro instances
     * });
     *
     * text.insert(0, "Hello");
     * loro.commit();
     *
     * // Later, when you want to stop listening:
     * unsubscribe();
     * ```
     *
     * @example
     * ```ts
     * const loro1 = new Loro();
     * const loro2 = new Loro();
     *
     * // Set up two-way sync
     * loro1.subscribeLocalUpdates((updates) => {
     *   loro2.import(updates);
     * });
     *
     * loro2.subscribeLocalUpdates((updates) => {
     *   loro1.import(updates);
     * });
     *
     * // Now changes in loro1 will be reflected in loro2 and vice versa
     * ```
     */
    subscribeLocalUpdates(f: (bytes: Uint8Array) => void): () => void
}

/**
 * Represents a `Delta` type which is a union of different operations that can be performed.
 *
 * @typeparam T - The data type for the `insert` operation.
 *
 * The `Delta` type can be one of three distinct shapes:
 *
 * 1. Insert Operation:
 *    - `insert`: The item to be inserted, of type T.
 *    - `attributes`: (Optional) A dictionary of attributes, describing styles in richtext
 *
 * 2. Delete Operation:
 *    - `delete`: The number of elements to delete.
 *
 * 3. Retain Operation:
 *    - `retain`: The number of elements to retain.
 *    - `attributes`: (Optional) A dictionary of attributes, describing styles in richtext
 */
export type Delta<T> =
  | {
    insert: T;
    attributes?: { [key in string]: {} };
    retain?: undefined;
    delete?: undefined;
  }
  | {
    delete: number;
    attributes?: undefined;
    retain?: undefined;
    insert?: undefined;
  }
  | {
    retain: number;
    attributes?: { [key in string]: {} };
    delete?: undefined;
    insert?: undefined;
  };

/**
 * The unique id of each operation.
 */
export type OpId = { peer: PeerID, counter: number };

/**
 * Change is a group of continuous operations
 */
export interface Change {
    peer: PeerID,
    counter: number,
    lamport: number,
    length: number,
    /**
     * The timestamp in seconds.
     *
     * [Unix time](https://en.wikipedia.org/wiki/Unix_time)
     * It is the number of seconds that have elapsed since 00:00:00 UTC on 1 January 1970.
     */
    timestamp: number,
    deps: OpId[],
    message: string | undefined,
}


/**
 * Data types supported by loro
 */
export type Value =
  | ContainerID
  | string
  | number
  | boolean
  | null
  | { [key: string]: Value }
  | Uint8Array
  | Value[];

export type UndoConfig = {
    mergeInterval?: number,
    maxUndoSteps?: number,
    excludeOriginPrefixes?: string[],
    onPush?: (isUndo: boolean, counterRange: { start: number, end: number }) => { value: Value, cursors: Cursor[] },
    onPop?: (isUndo: boolean, value: { value: Value, cursors: Cursor[] }, counterRange: { start: number, end: number }) => void
};
export type Container = LoroList | LoroMap | LoroText | LoroTree | LoroMovableList;

export interface ImportBlobMetadata {
    /**
     * The version vector of the start of the import.
     *
     * Import blob includes all the ops from `partial_start_vv` to `partial_end_vv`.
     * However, it does not constitute a complete version vector, as it only contains counters
     * from peers included within the import blob.
     */
    partialStartVersionVector: VersionVector;
    /**
     * The version vector of the end of the import.
     *
     * Import blob includes all the ops from `partial_start_vv` to `partial_end_vv`.
     * However, it does not constitute a complete version vector, as it only contains counters
     * from peers included within the import blob.
     */
    partialEndVersionVector: VersionVector;

    startFrontiers: OpId[],
    startTimestamp: number;
    endTimestamp: number;
    mode: "outdated-snapshot" | "outdated-update" | "snapshot" | "shallow-snapshot" | "update";
    changeNum: number;
}

interface LoroText {
    /**
     * Get the cursor position at the given pos.
     *
     * When expressing the position of a cursor, using "index" can be unstable
     * because the cursor's position may change due to other deletions and insertions,
     * requiring updates with each edit. To stably represent a position or range within
     * a list structure, we can utilize the ID of each item/character on List CRDT or
     * Text CRDT for expression.
     *
     * Loro optimizes State metadata by not storing the IDs of deleted elements. This
     * approach complicates tracking cursors since they rely on these IDs. The solution
     * recalculates position by replaying relevant history to update cursors
     * accurately. To minimize the performance impact of history replay, the system
     * updates cursor info to reference only the IDs of currently present elements,
     * thereby reducing the need for replay.
     *
     * @example
     * ```ts
     *
     * const doc = new LoroDoc();
     * const text = doc.getText("text");
     * text.insert(0, "123");
     * const pos0 = text.getCursor(0, 0);
     * {
     *   const ans = doc.getCursorPos(pos0!);
     *   expect(ans.offset).toBe(0);
     * }
     * text.insert(0, "1");
     * {
     *   const ans = doc.getCursorPos(pos0!);
     *   expect(ans.offset).toBe(1);
     * }
     * ```
     */
    getCursor(pos: number, side?: Side): Cursor | undefined;
}

interface LoroList {
    /**
     * Get the cursor position at the given pos.
     *
     * When expressing the position of a cursor, using "index" can be unstable
     * because the cursor's position may change due to other deletions and insertions,
     * requiring updates with each edit. To stably represent a position or range within
     * a list structure, we can utilize the ID of each item/character on List CRDT or
     * Text CRDT for expression.
     *
     * Loro optimizes State metadata by not storing the IDs of deleted elements. This
     * approach complicates tracking cursors since they rely on these IDs. The solution
     * recalculates position by replaying relevant history to update cursors
     * accurately. To minimize the performance impact of history replay, the system
     * updates cursor info to reference only the IDs of currently present elements,
     * thereby reducing the need for replay.
     *
     * @example
     * ```ts
     *
     * const doc = new LoroDoc();
     * const text = doc.getList("list");
     * text.insert(0, "1");
     * const pos0 = text.getCursor(0, 0);
     * {
     *   const ans = doc.getCursorPos(pos0!);
     *   expect(ans.offset).toBe(0);
     * }
     * text.insert(0, "1");
     * {
     *   const ans = doc.getCursorPos(pos0!);
     *   expect(ans.offset).toBe(1);
     * }
     * ```
     */
    getCursor(pos: number, side?: Side): Cursor | undefined;
}

export type TreeNodeValue = {
    id: TreeID,
    parent: TreeID | undefined,
    index: number,
    fractionalIndex: string,
    meta: LoroMap,
    children: TreeNodeValue[],
}

export type TreeNodeJSON<T> = Omit<TreeNodeValue, 'meta' | 'children'> & {
    meta: T,
    children: TreeNodeJSON<T>[],
}

interface LoroTree{
    toArray(): TreeNodeValue[];
    getNodes(options?: { withDeleted: boolean = false }): LoroTreeNode[];
}

interface LoroMovableList {
    /**
     * Get the cursor position at the given pos.
     *
     * When expressing the position of a cursor, using "index" can be unstable
     * because the cursor's position may change due to other deletions and insertions,
     * requiring updates with each edit. To stably represent a position or range within
     * a list structure, we can utilize the ID of each item/character on List CRDT or
     * Text CRDT for expression.
     *
     * Loro optimizes State metadata by not storing the IDs of deleted elements. This
     * approach complicates tracking cursors since they rely on these IDs. The solution
     * recalculates position by replaying relevant history to update cursors
     * accurately. To minimize the performance impact of history replay, the system
     * updates cursor info to reference only the IDs of currently present elements,
     * thereby reducing the need for replay.
     *
     * @example
     * ```ts
     *
     * const doc = new LoroDoc();
     * const text = doc.getMovableList("text");
     * text.insert(0, "1");
     * const pos0 = text.getCursor(0, 0);
     * {
     *   const ans = doc.getCursorPos(pos0!);
     *   expect(ans.offset).toBe(0);
     * }
     * text.insert(0, "1");
     * {
     *   const ans = doc.getCursorPos(pos0!);
     *   expect(ans.offset).toBe(1);
     * }
     * ```
     */
    getCursor(pos: number, side?: Side): Cursor | undefined;
}

export type Side = -1 | 0 | 1;
"#;

#[wasm_bindgen(typescript_custom_section)]
const JSON_SCHEMA_TYPES: &'static str = r#"
export type JsonOpID = `${number}@${PeerID}`;
export type JsonContainerID =  `🦜:${ContainerID}` ;
export type JsonValue  =
  | JsonContainerID
  | string
  | number
  | boolean
  | null
  | { [key: string]: JsonValue }
  | Uint8Array
  | JsonValue[];

export type JsonSchema = {
  schema_version: number;
  start_version: Map<string, number>,
  peers: PeerID[],
  changes: JsonChange[]
};

export type JsonChange = {
  id: JsonOpID
  /**
   * The timestamp in seconds.
   *
   * [Unix time](https://en.wikipedia.org/wiki/Unix_time)
   * It is the number of seconds that have elapsed since 00:00:00 UTC on 1 January 1970.
   */
  timestamp: number,
  deps: JsonOpID[],
  lamport: number,
  msg: string | null,
  ops: JsonOp[]
}

export interface TextUpdateOptions {
    timeoutMs?: number,
    useRefinedDiff?: boolean,
}

export type ExportMode = {
    mode: "update",
    from?: VersionVector,
} | {
    mode: "snapshot",
} | {
    mode: "shallow-snapshot",
    frontiers: Frontiers,
} | {
    mode: "updates-in-range",
    spans: {
        id: ID,
        len: number,
    }[],
};

export type JsonOp = {
  container: ContainerID,
  counter: number,
  content: ListOp | TextOp | MapOp | TreeOp | MovableListOp | UnknownOp
}

export type ListOp = {
  type: "insert",
  pos: number,
  value: JsonValue
} | {
  type: "delete",
  pos: number,
  len: number,
  start_id: JsonOpID,
};

export type MovableListOp = {
  type: "insert",
  pos: number,
  value: JsonValue
} | {
  type: "delete",
  pos: number,
  len: number,
  start_id: JsonOpID,
}| {
  type: "move",
  from: number,
  to: number,
  elem_id: JsonOpID,
}|{
  type: "set",
  elem_id: JsonOpID,
  value: JsonValue
}

export type TextOp = {
  type: "insert",
  pos: number,
  text: string
} | {
  type: "delete",
  pos: number,
  len: number,
  start_id: JsonOpID,
} | {
  type: "mark",
  start: number,
  end: number,
  style_key: string,
  style_value: JsonValue,
  info: number
}|{
  type: "mark_end"
};

export type MapOp = {
  type: "insert",
  key: string,
  value: JsonValue
} | {
  type: "delete",
  key: string,
};

export type TreeOp = {
  type: "create",
  target: TreeID,
  parent: TreeID | undefined,
  fractional_index: string
}|{
  type: "move",
  target: TreeID,
  parent: TreeID | undefined,
  fractional_index: string
}|{
  type: "delete",
  target: TreeID
};

export type UnknownOp = {
  type: "unknown"
  prop: number,
  value_type: "unknown",
  value: {
    kind: number,
    data: Uint8Array
  }
};

export type CounterSpan = { start: number, end: number };

export type ImportStatus = {
  success: Map<PeerID, CounterSpan>,
  pending: Map<PeerID, CounterSpan> | null
}

export type Frontiers = OpId[];

/**
 * Represents a path to identify the exact location of an event's target.
 * The path is composed of numbers (e.g., indices of a list container) strings
 * (e.g., keys of a map container) and TreeID (the node of a tree container),
 * indicating the absolute position of the event's source within a loro document.
 */
export type Path = (number | string | TreeID)[];

/**
 * A batch of events that created by a single `import`/`transaction`/`checkout`.
 *
 * @prop by - How the event is triggered.
 * @prop origin - (Optional) Provides information about the origin of the event.
 * @prop diff - Contains the differential information related to the event.
 * @prop target - Identifies the container ID of the event's target.
 * @prop path - Specifies the absolute path of the event's emitter, which can be an index of a list container or a key of a map container.
 */
export interface LoroEventBatch {
    /**
     * How the event is triggered.
     *
     * - `local`: The event is triggered by a local transaction.
     * - `import`: The event is triggered by an import operation.
     * - `checkout`: The event is triggered by a checkout operation.
     */
    by: "local" | "import" | "checkout";
    origin?: string;
    /**
     * The container ID of the current event receiver.
     * It's undefined if the subscriber is on the root document.
     */
    currentTarget?: ContainerID;
    events: LoroEvent[];
}

/**
 * The concrete event of Loro.
 */
export interface LoroEvent {
    /**
     * The container ID of the event's target.
     */
    target: ContainerID;
    diff: Diff;
    /**
     * The absolute path of the event's emitter, which can be an index of a list container or a key of a map container.
     */
    path: Path;
}

export type ListDiff = {
    type: "list";
    diff: Delta<(Value | Container)[]>[];
};

export type TextDiff = {
    type: "text";
    diff: Delta<string>[];
};

export type MapDiff = {
    type: "map";
    updated: Record<string, Value | Container | undefined>;
};

export type TreeDiffItem =
    | {
        target: TreeID;
        action: "create";
        parent: TreeID | undefined;
        index: number;
        fractionalIndex: string;
    }
    | {
        target: TreeID;
        action: "delete";
        oldParent: TreeID | undefined;
        oldIndex: number;
    }
    | {
        target: TreeID;
        action: "move";
        parent: TreeID | undefined;
        index: number;
        fractionalIndex: string;
        oldParent: TreeID | undefined;
        oldIndex: number;
    };

export type TreeDiff = {
    type: "tree";
    diff: TreeDiffItem[];
};

export type CounterDiff = {
    type: "counter";
    increment: number;
};

export type Diff = ListDiff | TextDiff | MapDiff | TreeDiff | CounterDiff;
export type Subscription = () => void;
type NonNullableType<T> = Exclude<T, null | undefined>;
export type AwarenessListener = (
    arg: { updated: PeerID[]; added: PeerID[]; removed: PeerID[] },
    origin: "local" | "timeout" | "remote" | string,
) => void;


interface Listener {
    (event: LoroEventBatch): void;
}

interface LoroDoc {
    subscribe(listener: Listener): Subscription;
}

interface UndoManager {
    /**
     * Set the callback function that is called when an undo/redo step is pushed.
     * The function can return a meta data value that will be attached to the given stack item.
     *
     * @param listener - The callback function.
     */
    setOnPush(listener?: UndoConfig["onPush"]): void;
    /**
     * Set the callback function that is called when an undo/redo step is popped.
     * The function will have a meta data value that was attached to the given stack item when `onPush` was called.
     *
     * @param listener - The callback function.
     */
    setOnPop(listener?: UndoConfig["onPop"]): void;
}
interface LoroDoc<T extends Record<string, Container> = Record<string, Container>> {
    /**
     * Get a LoroMap by container id
     *
     * The object returned is a new js object each time because it need to cross
     * the WASM boundary.
     *
     * @example
     * ```ts
     * import { LoroDoc } from "loro-crdt";
     *
     * const doc = new LoroDoc();
     * const map = doc.getMap("map");
     * ```
     */
    getMap<Key extends keyof T | ContainerID>(name: Key): T[Key] extends LoroMap ? T[Key] : LoroMap;
    /**
     * Get a LoroList by container id
     *
     * The object returned is a new js object each time because it need to cross
     * the WASM boundary.
     *
     * @example
     * ```ts
     * import { LoroDoc } from "loro-crdt";
     *
     * const doc = new LoroDoc();
     * const list = doc.getList("list");
     * ```
     */
    getList<Key extends keyof T | ContainerID>(name: Key): T[Key] extends LoroList ? T[Key] : LoroList;
    /**
     * Get a LoroMovableList by container id
     *
     * The object returned is a new js object each time because it need to cross
     * the WASM boundary.
     *
     * @example
     * ```ts
     * import { LoroDoc } from "loro-crdt";
     *
     * const doc = new LoroDoc();
     * const list = doc.getList("list");
     * ```
     */
    getMovableList<Key extends keyof T | ContainerID>(name: Key): T[Key] extends LoroMovableList ? T[Key] : LoroMovableList;
    /**
     * Get a LoroTree by container id
     *
     *  The object returned is a new js object each time because it need to cross
     *  the WASM boundary.
     *
     *  @example
     *  ```ts
     *  import { LoroDoc } from "loro-crdt";
     *
     *  const doc = new LoroDoc();
     *  const tree = doc.getTree("tree");
     *  ```
     */
    getTree<Key extends keyof T | ContainerID>(name: Key): T[Key] extends LoroTree ? T[Key] : LoroTree;
    getText(key: string | ContainerID): LoroText;
}
interface LoroList<T = unknown> {
    new(): LoroList<T>;
    /**
     *  Get elements of the list. If the value is a child container, the corresponding
     *  `Container` will be returned.
     *
     *  @example
     *  ```ts
     *  import { LoroDoc, LoroText } from "loro-crdt";
     *
     *  const doc = new LoroDoc();
     *  const list = doc.getList("list");
     *  list.insert(0, 100);
     *  list.insert(1, "foo");
     *  list.insert(2, true);
     *  list.insertContainer(3, new LoroText());
     *  console.log(list.value);  // [100, "foo", true, LoroText];
     *  ```
     */
    toArray(): T[];
    /**
     * Insert a container at the index.
     *
     *  @example
     *  ```ts
     *  import { LoroDoc, LoroText } from "loro-crdt";
     *
     *  const doc = new LoroDoc();
     *  const list = doc.getList("list");
     *  list.insert(0, 100);
     *  const text = list.insertContainer(1, new LoroText());
     *  text.insert(0, "Hello");
     *  console.log(list.toJSON());  // [100, "Hello"];
     *  ```
     */
    insertContainer<C extends Container>(pos: number, child: C): T extends C ? T : C;
    /**
     * Push a container to the end of the list.
     */
    pushContainer<C extends Container>(child: C): T extends C ? T : C;
    /**
     * Get the value at the index. If the value is a container, the corresponding handler will be returned.
     *
     *  @example
     *  ```ts
     *  import { LoroDoc } from "loro-crdt";
     *
     *  const doc = new LoroDoc();
     *  const list = doc.getList("list");
     *  list.insert(0, 100);
     *  console.log(list.get(0));  // 100
     *  console.log(list.get(1));  // undefined
     *  ```
     */
    get(index: number): T;
    /**
     *  Insert a value at index.
     *
     *  @example
     *  ```ts
     *  import { LoroDoc } from "loro-crdt";
     *
     *  const doc = new LoroDoc();
     *  const list = doc.getList("list");
     *  list.insert(0, 100);
     *  list.insert(1, "foo");
     *  list.insert(2, true);
     *  console.log(list.value);  // [100, "foo", true];
     *  ```
     */
    insert<V extends T>(pos: number, value: Exclude<V, Container>): void;
    delete(pos: number, len: number): void;
    push<V extends T>(value: Exclude<V, Container>): void;
    subscribe(listener: Listener): Subscription;
    getAttached(): undefined | LoroList<T>;
}
interface LoroMovableList<T = unknown> {
    new(): LoroMovableList<T>;
    /**
     *  Get elements of the list. If the value is a child container, the corresponding
     *  `Container` will be returned.
     *
     *  @example
     *  ```ts
     *  import { LoroDoc, LoroText } from "loro-crdt";
     *
     *  const doc = new LoroDoc();
     *  const list = doc.getMovableList("list");
     *  list.insert(0, 100);
     *  list.insert(1, "foo");
     *  list.insert(2, true);
     *  list.insertContainer(3, new LoroText());
     *  console.log(list.value);  // [100, "foo", true, LoroText];
     *  ```
     */
    toArray(): T[];
    /**
     * Insert a container at the index.
     *
     *  @example
     *  ```ts
     *  import { LoroDoc, LoroText } from "loro-crdt";
     *
     *  const doc = new LoroDoc();
     *  const list = doc.getMovableList("list");
     *  list.insert(0, 100);
     *  const text = list.insertContainer(1, new LoroText());
     *  text.insert(0, "Hello");
     *  console.log(list.toJSON());  // [100, "Hello"];
     *  ```
     */
    insertContainer<C extends Container>(pos: number, child: C): T extends C ? T : C;
    /**
     * Push a container to the end of the list.
     */
    pushContainer<C extends Container>(child: C): T extends C ? T : C;
    /**
     * Get the value at the index. If the value is a container, the corresponding handler will be returned.
     *
     *  @example
     *  ```ts
     *  import { LoroDoc } from "loro-crdt";
     *
     *  const doc = new LoroDoc();
     *  const list = doc.getMovableList("list");
     *  list.insert(0, 100);
     *  console.log(list.get(0));  // 100
     *  console.log(list.get(1));  // undefined
     *  ```
     */
    get(index: number): T;
    /**
     *  Insert a value at index.
     *
     *  @example
     *  ```ts
     *  import { LoroDoc } from "loro-crdt";
     *
     *  const doc = new LoroDoc();
     *  const list = doc.getMovableList("list");
     *  list.insert(0, 100);
     *  list.insert(1, "foo");
     *  list.insert(2, true);
     *  console.log(list.value);  // [100, "foo", true];
     *  ```
     */
    insert<V extends T>(pos: number, value: Exclude<V, Container>): void;
    delete(pos: number, len: number): void;
    push<V extends T>(value: Exclude<V, Container>): void;
    subscribe(listener: Listener): Subscription;
    getAttached(): undefined | LoroMovableList<T>;
    /**
     *  Set the value at the given position.
     *
     *  It's different from `delete` + `insert` that it will replace the value at the position.
     *
     *  For example, if you have a list `[1, 2, 3]`, and you call `set(1, 100)`, the list will be `[1, 100, 3]`.
     *  If concurrently someone call `set(1, 200)`, the list will be `[1, 200, 3]` or `[1, 100, 3]`.
     *
     *  But if you use `delete` + `insert` to simulate the set operation, they may create redundant operations
     *  and the final result will be `[1, 100, 200, 3]` or `[1, 200, 100, 3]`.
     *
     *  @example
     *  ```ts
     *  import { LoroDoc } from "loro-crdt";
     *
     *  const doc = new LoroDoc();
     *  const list = doc.getMovableList("list");
     *  list.insert(0, 100);
     *  list.insert(1, "foo");
     *  list.insert(2, true);
     *  list.set(1, "bar");
     *  console.log(list.value);  // [100, "bar", true];
     *  ```
     */
    set<V extends T>(pos: number, value: Exclude<V, Container>): void;
    /**
     * Set a container at the index.
     *
     *  @example
     *  ```ts
     *  import { LoroDoc, LoroText } from "loro-crdt";
     *
     *  const doc = new LoroDoc();
     *  const list = doc.getMovableList("list");
     *  list.insert(0, 100);
     *  const text = list.setContainer(0, new LoroText());
     *  text.insert(0, "Hello");
     *  console.log(list.toJSON());  // ["Hello"];
     *  ```
     */
    setContainer<C extends Container>(pos: number, child: C): T extends C ? T : C;
}
interface LoroMap<T extends Record<string, unknown> = Record<string, unknown>> {
    new(): LoroMap<T>;
    /**
     *  Get the value of the key. If the value is a child container, the corresponding
     *  `Container` will be returned.
     *
     *  The object returned is a new js object each time because it need to cross
     *
     *  @example
     *  ```ts
     *  import { LoroDoc } from "loro-crdt";
     *
     *  const doc = new LoroDoc();
     *  const map = doc.getMap("map");
     *  map.set("foo", "bar");
     *  const bar = map.get("foo");
     *  ```
     */
    getOrCreateContainer<C extends Container>(key: string, child: C): C;
    /**
     * Set the key with a container.
     *
     *  @example
     *  ```ts
     *  import { LoroDoc, LoroText, LoroList } from "loro-crdt";
     *
     *  const doc = new LoroDoc();
     *  const map = doc.getMap("map");
     *  map.set("foo", "bar");
     *  const text = map.setContainer("text", new LoroText());
     *  const list = map.setContainer("list", new LoroList());
     *  ```
     */
    setContainer<C extends Container, Key extends keyof T>(key: Key, child: C): NonNullableType<T[Key]> extends C ? NonNullableType<T[Key]> : C;
    /**
     *  Get the value of the key. If the value is a child container, the corresponding
     *  `Container` will be returned.
     *
     *  The object/value returned is a new js object/value each time because it need to cross
     *  the WASM boundary.
     *
     *  @example
     *  ```ts
     *  import { LoroDoc } from "loro-crdt";
     *
     *  const doc = new LoroDoc();
     *  const map = doc.getMap("map");
     *  map.set("foo", "bar");
     *  const bar = map.get("foo");
     *  ```
     */
    get<Key extends keyof T>(key: Key): T[Key];
    /**
     * Set the key with the value.
     *
     *  If the value of the key is exist, the old value will be updated.
     *
     *  @example
     *  ```ts
     *  import { LoroDoc } from "loro-crdt";
     *
     *  const doc = new LoroDoc();
     *  const map = doc.getMap("map");
     *  map.set("foo", "bar");
     *  map.set("foo", "baz");
     *  ```
     */
    set<Key extends keyof T, V extends T[Key]>(key: Key, value: Exclude<V, Container>): void;
    delete(key: string): void;
    subscribe(listener: Listener): Subscription;
}
interface LoroText {
    new(): LoroText;
    insert(pos: number, text: string): void;
    delete(pos: number, len: number): void;
    subscribe(listener: Listener): Subscription;
    /**
     * Update the current text to the target text.
     *
     * It will calculate the minimal difference and apply it to the current text.
     * It uses Myers' diff algorithm to compute the optimal difference.
     *
     * This could take a long time for large texts (e.g. > 50_000 characters).
     * In that case, you should use `updateByLine` instead.
     *
     * @example
     * ```ts
     * import { LoroDoc } from "loro-crdt";
     *
     * const doc = new LoroDoc();
     * const text = doc.getText("text");
     * text.insert(0, "Hello");
     * text.update("Hello World");
     * console.log(text.toString()); // "Hello World"
     * ```
     */
    update(text: string, options?: TextUpdateOptions): void;
    /**
     * Update the current text based on the provided text.
     * This update calculation is line-based, which will be more efficient but less precise.
     */
    updateByLine(text: string, options?: TextUpdateOptions): void;
}
interface LoroTree<T extends Record<string, unknown> = Record<string, unknown>> {
    new(): LoroTree<T>;
    /**
     * Create a new tree node as the child of parent and return a `LoroTreeNode` instance.
     * If the parent is undefined, the tree node will be a root node.
     *
     * If the index is not provided, the new node will be appended to the end.
     *
     * @example
     * ```ts
     * import { LoroDoc } from "loro-crdt";
     *
     * const doc = new LoroDoc();
     * const tree = doc.getTree("tree");
     * const root = tree.createNode();
     * const node = tree.createNode(undefined, 0);
     *
     * //  undefined
     * //    /   \
     * // node  root
     * ```
     */
    createNode(parent?: TreeID, index?: number): LoroTreeNode<T>;
    move(target: TreeID, parent?: TreeID, index?: number): void;
    delete(target: TreeID): void;
    has(target: TreeID): boolean;
    /**
     * Get LoroTreeNode by the TreeID.
     */
    getNodeByID(target: TreeID): LoroTreeNode<T>;
    subscribe(listener: Listener): Subscription;
}
interface LoroTreeNode<T extends Record<string, unknown> = Record<string, unknown>> {
    /**
     * Get the associated metadata map container of a tree node.
     */
    readonly data: LoroMap<T>;
    /**
     * Create a new node as the child of the current node and
     * return an instance of `LoroTreeNode`.
     *
     * If the index is not provided, the new node will be appended to the end.
     *
     * @example
     * ```typescript
     * import { LoroDoc } from "loro-crdt";
     *
     * let doc = new LoroDoc();
     * let tree = doc.getTree("tree");
     * let root = tree.createNode();
     * let node = root.createNode();
     * let node2 = root.createNode(0);
     * //    root
     * //    /  \
     * // node2 node
     * ```
     */
    createNode(index?: number): LoroTreeNode<T>;
    move(parent?: LoroTreeNode<T>, index?: number): void;
    parent(): LoroTreeNode<T> | undefined;
    /**
     * Get the children of this node.
     *
     * The objects returned are new js objects each time because they need to cross
     * the WASM boundary.
     */
    children(): Array<LoroTreeNode<T>> | undefined;
    toJSON(): TreeNodeJSON<T>;
}
interface AwarenessWasm<T extends Value = Value> {
    getState(peer: PeerID): T | undefined;
    getTimestamp(peer: PeerID): number | undefined;
    getAllStates(): Record<PeerID, T>;
    setLocalState(value: T): void;
    removeOutdated(): PeerID[];
}

"#;
