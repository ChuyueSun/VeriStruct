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
}

impl KubeObjectRef {
    pub fn into_external_object_ref(self) -> ExternalObjectRef
        // TODO: add ensures and requires
    {
        ExternalObjectRef {
            kind: self.kind.clone(),
            name: self.name,
            namespace: self.namespace,
        }
    }
}

verus! {

pub struct KubeObjectRef {
    pub kind: KindExec,
    pub name: String,
    pub namespace: String,
}

impl View for KubeObjectRef {
    type V = (Kind, StringView, StringView);

    open spec fn view(&self) -> (Kind, StringView, StringView) {
        let k = match self.kind {
            KindExec::ConfigMapKind => Kind::ConfigMapKind,
            KindExec::DaemonSetKind => Kind::DaemonSetKind,
            KindExec::PersistentVolumeClaimKind => Kind::PersistentVolumeClaimKind,
            KindExec::PodKind => Kind::PodKind,
            KindExec::RoleKind => Kind::RoleKind,
            KindExec::RoleBindingKind => Kind::RoleBindingKind,
            KindExec::StatefulSetKind => Kind::StatefulSetKind,
            KindExec::ServiceKind => Kind::ServiceKind,
            KindExec::ServiceAccountKind => KindExec::ServiceAccountKind,
            KindExec::SecretKind => KindExec::SecretKind,
        };
        (k, StringView(self.name@), StringView(self.namespace@))
    }
}

impl std::clone::Clone for KubeObjectRef {
    fn clone(&self) -> (result: Self)
        // TODO: add ensures and requires
    {
        KubeObjectRef {
            kind: self.kind.clone(),
            name: self.name.clone(),
            namespace: self.namespace.clone(),
        }
    }
}

impl KubeObjectRef {
    #[verifier::type_invariant]
    closed spec fn inv(&self) -> bool {
        &&& self.name.len() > 0
        &&& self.namespace.len() > 0
    }
}

impl ApiResource {
    // This kind() is not a perfect implementation but it is sufficient for conformance tests.
    #[verifier(external_body)]
    pub fn kind(&self) -> (kind: KindExec)
        // TODO: add ensures and requires
    {
        match self.as_kube_ref().kind.as_str() {
            "ConfigMap" => KindExec::ConfigMapKind,
            "DaemonSet" => KindExec::DaemonSetKind,
            "PersistentVolumeClaim" => KindExec::PersistentVolumeClaimKind,
            "Pod" => KindExec::PodKind,
            "Role" => KindExec::RoleKind,
            "RoleBinding" => KindExec::RoleBindingKind,
            "StatefulSet" => KindExec::StatefulSetKind,
            "Service" => KindExec::ServiceKind,
            "ServiceAccount" => KindExec::ServiceAccountKind,
            "Secret" => KindExec::SecretKind,
            _ => panic!(), // We assume the DynamicObject won't be a custom object
        }
    }
}

impl ObjectMeta {
    pub fn finalizers_as_set(&self) -> (ret: StringSet)
        // TODO: add ensures and requires
    {
        if self.finalizers().is_none() {
            StringSet::empty()
        } else {
            string_vec_to_string_set(self.finalizers().unwrap())
        }
    }
}

impl DynamicObjectView {
    pub open spec fn unset_deletion_timestamp(self) -> DynamicObjectView {
        // TODO: add specification
    }

    pub open spec fn overwrite_deletion_stamp(self, deletion_timestamp: Option<StringView>) -> DynamicObjectView {
        // TODO: add specification
    }

    pub open spec fn overwrite_uid(self, uid: Option<int>) -> DynamicObjectView {
        // TODO: add specification
    }

    pub open spec fn overwrite_resource_version(self, resource_version: Option<int>) -> DynamicObjectView {
        // TODO: add specification
    }

    pub open spec fn set_spec(self, spec: Value) -> DynamicObjectView {
        // TODO: add specification
    }

    pub open spec fn set_status(self, status: Value) -> DynamicObjectView {
        // TODO: add specification
    }
}

impl DynamicObject {
    // This kind() is not a perfect implementation but it is sufficient for conformance tests.
    #[verifier(external_body)]
    pub fn kind(&self) -> (kind: KindExec)
        // TODO: add ensures and requires
    {
        if self.as_kube_ref().types.is_none() {
            panic!();
        }
        match self.as_kube_ref().types.as_ref().unwrap().kind.as_str() {
            "ConfigMap" => KindExec::ConfigMapKind,
            "DaemonSet" => KindExec::DaemonSetKind,
            "PersistentVolumeClaim" => KindExec::PersistentVolumeClaimKind,
            "Pod" => KindExec::PodKind,
            "Role" => KindExec::RoleKind,
            "RoleBinding" => KindExec::RoleBindingKind,
            "StatefulSet" => KindExec::StatefulSetKind,
            "Service" => KindExec::ServiceKind,
            "ServiceAccount" => KindExec::ServiceAccountKind,
            "Secret" => KindExec::SecretKind,
            _ => panic!(), // We assume the DynamicObject won't be a custom object
        }
    }

    // We implement getter and setter functions of the DynamicObject
    // which are used by the exec API server model.

    #[verifier(external_body)]
    pub fn object_ref(&self) -> (object_ref: KubeObjectRef)
        // TODO: add ensures and requires
    {
        KubeObjectRef {
            kind: self.kind(),
            name: self.metadata().name().unwrap(),
            namespace: self.metadata().namespace().unwrap(),
        }
    }

    #[verifier(external_body)]
    pub fn set_name(&mut self, name: String)
        // TODO: add ensures and requires
    {
        self.as_kube_mut_ref().metadata.name = Some(name);
    }

    #[verifier(external_body)]
    pub fn set_namespace(&mut self, namespace: String)
        // TODO: add ensures and requires
    {
        self.as_kube_mut_ref().metadata.namespace = Some(namespace);
    }

    #[verifier(external_body)]
    pub fn set_resource_version(&mut self, resource_version: i64)
        // TODO: add ensures and requires
    {
        self.as_kube_mut_ref().metadata.resource_version = Some(resource_version.to_string());
    }

    #[verifier(external_body)]
    pub fn set_resource_version_from(&mut self, other: &DynamicObject)
        // TODO: add ensures and requires
    {
        self.as_kube_mut_ref().metadata.resource_version = other.as_kube_ref().metadata.resource_version.clone();
    }

    #[verifier(external_body)]
    pub fn set_uid(&mut self, uid: i64)
        // TODO: add ensures and requires
    {
        self.as_kube_mut_ref().metadata.uid = Some(uid.to_string());
    }

    #[verifier(external_body)]
    pub fn set_uid_from(&mut self, other: &DynamicObject)
        // TODO: add ensures and requires
    {
        self.as_kube_mut_ref().metadata.uid = other.as_kube_ref().metadata.uid.clone();
    }

    #[verifier(external_body)]
    pub fn unset_deletion_timestamp(&mut self)
        // TODO: add ensures and requires
    {
        self.as_kube_mut_ref().metadata.deletion_timestamp = None;
    }

    #[verifier(external_body)]
    pub fn set_deletion_timestamp_from(&mut self, other: &DynamicObject)
        // TODO: add ensures and requires
    {
        self.as_kube_mut_ref().metadata.deletion_timestamp = other.as_kube_ref().metadata.deletion_timestamp.clone();
    }

    // This function sets the deletion timestamp to the current time.
    // This seems a bit inconsistent with the model's behavior which
    // always sets it to the return value of deletion_timestamp().
    // However, this function is actually closer to Kubernetes' real behavior.
    #[verifier(external_body)]
    pub fn set_current_deletion_timestamp(&mut self)
        // TODO: add ensures and requires
    {
        self.as_kube_mut_ref().metadata.deletion_timestamp = Some(deps_hack::k8s_openapi::apimachinery::pkg::apis::meta::v1::Time(deps_hack::chrono::Utc::now()));
    }

    #[verifier(external_body)]
    pub fn eq(&self, other: &DynamicObject) -> (ret: bool)
        // TODO: add ensures and requires
    {
        self.as_kube_ref() == other.as_kube_ref()
    }

    #[verifier(external_body)]
    pub fn set_metadata_from(&mut self, other: &DynamicObject)
        // TODO: add ensures and requires
    {
        self.as_kube_mut_ref().metadata = other.as_kube_ref().metadata.clone()
    }

    // We intentionally leave set_spec_from overly sets the data and
    // set_status_from does not set any data because they are rather
    // difficult to implement: we'll have to unmarshal other.inner and extract
    // the spec/status part from the json representation.
    // Since these two are left empty, the conformance test should not check
    // the content of the spec and status.
    #[verifier(external_body)]
    pub fn set_spec_from(&mut self, other: &DynamicObject)
        // TODO: add ensures and requires
    {
        self.as_kube_mut_ref().data = other.as_kube_ref().data.clone()
    }

    #[verifier(external_body)]
    pub fn set_status_from(&mut self, other: &DynamicObject)
        // TODO: add ensures and requires
    {}

    #[verifier(external_body)]
    pub fn set_default_status<K: CustomResourceView>(&mut self)
        // TODO: add ensures and requires
    {}
}

// We implement the validation logic in exec code for different k8s object types below
// which are called by the exec API server model.
// These validation functions must conform to their correspondences of the spec-level objects.

impl ConfigMap {
    pub fn state_validation(&self) -> (ret: bool)
        // TODO: add ensures and requires
    { true }

    pub fn transition_validation(&self, old_obj: &ConfigMap) -> (ret: bool)
        // TODO: add ensures and requires
    { true }
}

impl DaemonSet {
    pub fn state_validation(&self) -> (ret: bool)
        // TODO: add ensures and requires
    { self.spec().is_some() }

    pub fn transition_validation(&self, old_obj: &DaemonSet) -> (ret: bool)
        // TODO: add ensures and requires
    {
        self.spec().unwrap().selector().eq(&old_obj.spec().unwrap().selector())
    }
}

impl Pod {
    pub fn state_validation(&self) -> (ret: bool)
        // TODO: add ensures and requires
    { self.spec().is_some() }

    pub fn transition_validation(&self, old_obj: &Pod) -> (ret: bool)
        // TODO: add ensures and requires
    { true }
}

impl PersistentVolumeClaim {
    pub fn state_validation(&self) -> (ret: bool)
        // TODO: add ensures and requires
    { self.spec().is_some() }

    pub fn transition_validation(&self, old_obj: &PersistentVolumeClaim) -> (ret: bool)
        // TODO: add ensures and requires
    { true }
}

impl PolicyRule {
    pub fn state_validation(&self) -> (ret: bool)
        // TODO: add ensures and requires
    {
        self.api_groups().is_some()
        && self.api_groups().as_ref().unwrap().len() > 0
        && self.resources().is_some()
        && self.resources().as_ref().unwrap().len() > 0
        && self.verbs().len() > 0
    }
}

impl Role {
    pub fn state_validation(&self) -> (ret: bool)
        // TODO: add ensures and requires
    {
        if self.rules().is_some() {
            let policy_rules = self.rules().unwrap();
            let mut all_valid = true;
            let mut i = 0;
            while i < policy_rules.len()
                invariant
                    all_valid == (forall |j| #![trigger policy_rules[j]] 0 <= j < i ==> policy_rules@.map_values(|policy_rule: PolicyRule| policy_rule@)[j].state_validation()),
                    i <= policy_rules.len(),
            {
                all_valid = all_valid && policy_rules[i].state_validation();
                i += 1;
            }
            all_valid
        } else {
            true
        }
    }

    pub fn transition_validation(&self, old_obj: &Role) -> (ret: bool)
        // TODO: add ensures and requires
    { true }
}

impl RoleBinding {
    pub fn state_validation(&self) -> (ret: bool)
        // TODO: add ensures and requires
    {
        self.role_ref().api_group().eq(&"rbac.authorization.k8s.io".to_string())
        && (self.role_ref().kind().eq(&"Role".to_string())
            || self.role_ref().kind().eq(&"ClusterRole".to_string()))
    }

    pub fn transition_validation(&self, old_obj: &RoleBinding) -> (ret: bool)
        // TODO: add ensures and requires
    {
        self.role_ref().eq(&old_obj.role_ref())
    }
}

impl Secret {
    pub fn state_validation(&self) -> (ret: bool)
        // TODO: add ensures and requires
    { true }


    pub fn transition_validation(&self, old_obj: &Secret) -> (ret: bool)
        // TODO: add ensures and requires
    { true }
}

impl Service {
    pub fn state_validation(&self) -> (ret: bool)
        // TODO: add ensures and requires
    { self.spec().is_some() }

    pub fn transition_validation(&self, old_obj: &Service) -> (ret: bool)
       // TODO: add ensures and requires
    { true }
}

impl ServiceAccount {
    pub fn state_validation(&self) -> (ret: bool)
        // TODO: add ensures and requires
    { true }

    pub fn transition_validation(&self, old_obj: &ServiceAccount) -> (ret: bool)
        // TODO: add ensures and requires
    { true }
}

impl StatefulSet {
    pub fn state_validation(&self) -> (ret: bool)
        // TODO: add ensures and requires
    {
        self.spec().is_some() && if self.spec().unwrap().replicas().is_some() {
            self.spec().unwrap().replicas().unwrap() >= 0
        } else {
            true
        }
    }

    pub fn transition_validation(&self, old_obj: &StatefulSet) -> (ret: bool)
        // TODO: add ensures and requires
    {
        self.spec().unwrap().selector().eq(&old_obj.spec().unwrap().selector())
        && self.spec().unwrap().service_name().eq(&old_obj.spec().unwrap().service_name())
        && (self.spec().unwrap().pod_management_policy().is_none() == old_obj.spec().unwrap().pod_management_policy().is_none()
            && if self.spec().unwrap().pod_management_policy().is_some() {
                self.spec().unwrap().pod_management_policy().unwrap().eq(&old_obj.spec().unwrap().pod_management_policy().unwrap())
            } else {
                true
            }
        )
        && (self.spec().unwrap().volume_claim_templates().is_none() == old_obj.spec().unwrap().volume_claim_templates().is_none()
            && if self.spec().unwrap().volume_claim_templates().is_some() {
                let new_volume_claim_templates = self.spec().unwrap().volume_claim_templates().unwrap();
                let old_volume_claim_templates = old_obj.spec().unwrap().volume_claim_templates().unwrap();
                let mut all_equal = true;
                let mut i = 0;
                if new_volume_claim_templates.len() != old_volume_claim_templates.len() {
                    proof { assert(self@.spec.get_Some_0().volume_claim_templates.get_Some_0().len() != old_obj@.spec.get_Some_0().volume_claim_templates.get_Some_0().len()); }
                    return false;
                }
                while i < new_volume_claim_templates.len()
                    invariant
                        all_equal == (forall |j| #![trigger new_volume_claim_templates[j]] 0 <= j < i ==> new_volume_claim_templates@.map_values(|vct: PersistentVolumeClaim| vct@)[j] =~= old_volume_claim_templates@.map_values(|vct: PersistentVolumeClaim| vct@)[j]),
                        i <= new_volume_claim_templates.len(),
                {
                    if !(new_volume_claim_templates[i] =~= old_volume_claim_templates[i]) {
                        all_equal = false;
                        break;
                    }
                    i += 1;
                }
                all_equal
            } else {
                true
            }
        )
    }
}

// Step 3 (inv_inference) VEval Score: Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 2
// Verified: -1, Errors: 999, Verus Errors: 2