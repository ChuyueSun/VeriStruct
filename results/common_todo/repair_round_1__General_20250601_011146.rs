use crate::executable_model::string_set::*;
use crate::kubernetes_api_objects::error::UnmarshalError;
use crate::kubernetes_api_objects::exec::{api_resource::ApiResource, prelude::*};
use crate::kubernetes_api_objects::spec::prelude::*;
use crate::kubernetes_cluster::spec::{
    api_server::state_machine as model, api_server::types as model_types,
};
use crate::vstd_ext::string_view::StringView;
use vstd::prelude::*;
use vstd::string::*;

// We use ExternalObjectRef, instead of KubeObjectRef, as the key of the ObjectMap
// because the key has to implement a few traits including Ord and PartialOrd.
// It's easy to implement such traits for ExternalObjectRef but hard for KubeObjectRef
// because it internally uses vstd::string::String, which does not implement such traits.
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct ExternalObjectRef {
    pub kind: KindExec,
    pub name: std::string::String,
    pub namespace: std::string::String,


// Repair Round 1 VEval Score: Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 2
// Verified: -1, Errors: 999, Verus Errors: 2