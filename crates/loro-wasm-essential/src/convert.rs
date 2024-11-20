use std::sync::Arc;

use js_sys::{Array, Map, Object, Reflect, Uint8Array};
use loro_internal::delta::ResolvedMapDelta;
use loro_internal::encoding::{ImportBlobMetadata, ImportStatus};
use loro_internal::event::Diff;
use loro_internal::handler::{Handler, ValueOrHandler};
use loro_internal::version::VersionRange;
use loro_internal::{CounterSpan, ListDiffItem, LoroDoc, LoroValue};
use wasm_bindgen::JsValue;

use crate::{frontiers_to_ids, JsContainer, JsImportBlobMetadata, VersionVector};
use wasm_bindgen::__rt::IntoJsResult;
use wasm_bindgen::convert::RefFromWasmAbi;

pub(crate) fn js_to_version_vector(
    js: JsValue,
) -> Result<wasm_bindgen::__rt::Ref<'static, VersionVector>, JsValue> {
    if !js.is_object() {
        return Err(JsValue::from_str(&format!(
            "Value supplied is not an object, but {:?}",
            js
        )));
    }

    if js.is_null() || js.is_undefined() {
        return Err(JsValue::from_str(&format!(
            "Value supplied is not an object, but {:?}",
            js
        )));
    }

    if !js.is_object() {
        return Err(JsValue::from_str("Expected an object or Uint8Array"));
    }

    let Ok(ptr) = Reflect::get(&js, &JsValue::from_str("__wbg_ptr")) else {
        return Err(JsValue::from_str("Cannot find pointer field"));
    };

    let ptr_u32: u32 = ptr.as_f64().unwrap() as u32;
    let vv = unsafe { VersionVector::ref_from_abi(ptr_u32) };
    Ok(vv)
}

pub fn convert(value: LoroValue) -> JsValue {
    match value {
        LoroValue::Null => JsValue::NULL,
        LoroValue::Bool(b) => JsValue::from_bool(b),
        LoroValue::Double(f) => JsValue::from_f64(f),
        LoroValue::I64(i) => JsValue::from_f64(i as f64),
        LoroValue::String(s) => JsValue::from_str(&s),
        LoroValue::List(list) => {
            let list = list.unwrap();
            let arr = Array::new_with_length(list.len() as u32);
            for (i, v) in list.into_iter().enumerate() {
                arr.set(i as u32, convert(v));
            }
            arr.into_js_result().unwrap()
        }
        LoroValue::Map(m) => {
            let m = m.unwrap();
            let map = Object::new();
            for (k, v) in m.into_iter() {
                let str: &str = &k;
                js_sys::Reflect::set(&map, &JsValue::from_str(str), &convert(v)).unwrap();
            }

            map.into_js_result().unwrap()
        }
        LoroValue::Container(container_id) => JsValue::from(&container_id),
        LoroValue::Binary(binary) => {
            let binary = binary.unwrap();
            let arr = Uint8Array::new_with_length(binary.len() as u32);
            for (i, v) in binary.into_iter().enumerate() {
                arr.set_index(i as u32, v);
            }
            arr.into_js_result().unwrap()
        }
    }
}

impl From<ImportBlobMetadata> for JsImportBlobMetadata {
    fn from(meta: ImportBlobMetadata) -> Self {
        let start_vv = super::VersionVector(meta.partial_start_vv);
        let end_vv = super::VersionVector(meta.partial_end_vv);
        let start_vv: JsValue = start_vv.into();
        let end_vv: JsValue = end_vv.into();
        let start_timestamp: JsValue = JsValue::from_f64(meta.start_timestamp as f64);
        let end_timestamp: JsValue = JsValue::from_f64(meta.end_timestamp as f64);
        let mode: JsValue = JsValue::from_str(&meta.mode.to_string());
        let change_num: JsValue = JsValue::from_f64(meta.change_num as f64);
        let ans = Object::new();
        js_sys::Reflect::set(
            &ans,
            &JsValue::from_str("partialStartVersionVector"),
            &start_vv,
        )
        .unwrap();
        js_sys::Reflect::set(&ans, &JsValue::from_str("partialEndVersionVector"), &end_vv).unwrap();
        let js_frontiers: JsValue = frontiers_to_ids(&meta.start_frontiers).into();
        js_sys::Reflect::set(&ans, &JsValue::from_str("startFrontiers"), &js_frontiers).unwrap();
        js_sys::Reflect::set(&ans, &JsValue::from_str("startTimestamp"), &start_timestamp).unwrap();
        js_sys::Reflect::set(&ans, &JsValue::from_str("endTimestamp"), &end_timestamp).unwrap();
        js_sys::Reflect::set(&ans, &JsValue::from_str("mode"), &mode).unwrap();
        js_sys::Reflect::set(&ans, &JsValue::from_str("changeNum"), &change_num).unwrap();
        let ans: JsValue = ans.into();
        ans.into()
    }
}

pub(crate) fn import_status_to_js_value(status: ImportStatus) -> JsValue {
    let obj = Object::new();
    js_sys::Reflect::set(
        &obj,
        &JsValue::from_str("success"),
        &id_span_vector_to_js_value(status.success),
    )
    .unwrap();
    js_sys::Reflect::set(
        &obj,
        &JsValue::from_str("pending"),
        &match status.pending {
            None => JsValue::null(),
            Some(pending) => id_span_vector_to_js_value(pending),
        },
    )
    .unwrap();
    obj.into()
}

fn id_span_vector_to_js_value(v: VersionRange) -> JsValue {
    let map = Map::new();
    for (k, v) in v.iter() {
        Map::set(
            &map,
            &JsValue::from_str(&k.to_string()),
            &JsValue::from(CounterSpan {
                start: v.0,
                end: v.1,
            }),
        );
    }
    map.into()
}
