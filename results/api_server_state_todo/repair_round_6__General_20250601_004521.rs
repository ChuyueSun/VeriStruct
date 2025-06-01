use executable_model::{object_map::ObjectMap, object_ref_set::ObjectRefSet};
use kubernetes_api_objects::exec::dynamic::DynamicObject;
use kubernetes_api_objects::spec::{
    common::{Kind, ObjectRef},
    dynamic::{DynamicObjectView, StoredState},
};
use kubernetes_cluster::spec::api_server::types as model_types;
use vstd::prelude::*;
use vstd::string::*;

verus! {

// This is the exec version of crate::kubernetes_cluster::spec::api_server::types::ApiServerState
// and is used as the "state" of the exec API server model.
pub struct ApiServerState {
    pub resources: ObjectMap,
    pub uid_counter: i64,
    pub resource_version_counter: i64,
    pub stable_resources: ObjectRefSet,
}

impl ApiServerState {
    pub fn new() -> ApiServerState {
        ApiServerState {
            resources: ObjectMap::new(),
            uid_counter: 0,
            resource_version_counter: 0,
            stable_resources: ObjectRefSet::new(),
        }
    }
}

impl View for ApiServerState {
    type V = model_types::ApiServerState;
    open spec fn view(&self) -> model_types::ApiServerState {
        // TODO: implement specification.
    }
}

}

// Repair Round 6 VEval Score: Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 4
// Verified: -1, Errors: 999, Verus Errors: 4