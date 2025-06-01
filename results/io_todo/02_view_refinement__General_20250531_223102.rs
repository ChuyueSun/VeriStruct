use crate::kubernetes_api_objects::exec::api_method::*;
use crate::reconciler::spec::io::*;
use vstd::prelude::*;

use vstd::pervasive::unreached;

verus! {

pub struct VoidEReqView;

pub struct VoidERespView;

pub struct VoidEReq {}

impl View for VoidEReq {
    type V = ();

    spec fn view(&self) -> () {
        ()
    }
}

pub struct VoidEResp {}

impl View for VoidEResp {
    type V = ();

    spec fn view(&self) -> () {
        ()
    }
}

// Third-party libraries can also receive requests from reconciler.
// T: The input type of the third-party library of the reconciler which should also be defined by the developer.
// Typically, T can be an enum type, which lists all the possible supporting handlings the developer need support from the
// third-party library on.
// Then in the process method of the library, the developers can do pattern matching to generate desired output which will
// then be sent to the reconciler in the next-round reconcile loop.
// In reconcile_core, if the reconciler wants kubernetes to process the request, it should return a Request::KRequest;
// if it wants the third-party library to deal with the request, it should return a Request::ExternalRequest.
pub enum Request<T: View> {
    KRequest(KubeAPIRequest),
    ExternalRequest(T),
}

// The response type should be consistent with the Request type.
// T: the output type of the third-party library of the reconciler which should be defined by the developer.
// A safe and common way is to have an enum type which has the corresponding types of the input type in the Request.
// Anyway, the process method in the ExternalAPI, the input type in Request, output type in Response and the handling
// of external response in reconcile_core are correlative.
// Developers have the freedom to define them in their own preferred way as long as they make them work well.
#[is_variant]
pub enum Response<T: View> {
    KResponse(KubeAPIResponse),
    ExternalResponse(T),
}

// Views for the request and response:
pub enum RequestView<TV> {
    KRequest(KubeAPIRequestView),
    ExternalRequest(TV),
}

pub enum ResponseView<TV> {
    KResponse(KubeAPIResponseView),
    ExternalResponse(TV),
}

impl <T: View> View for Response<T> {
    type V = (Option<KubeAPIResponseView>, Option<T::V>);

    open spec fn view(&self) -> (Option<KubeAPIResponseView>, Option<T::V>) {
        match self {
            Response::KResponse(r) => (Some(r.view()), None),
            Response::ExternalResponse(e) => (None, Some(e.view())),
        }
    }
}

impl <T: View> Response<T> {
    pub fn is_external_response(&self) -> (res: bool)
    {
        match self {
            Response::ExternalResponse(_) => true,
            _ => false,
        }
    }

    pub fn as_external_response_ref(&self) -> (resp: &T)
    {
        match self {
            Response::ExternalResponse(resp) => resp,
            _ => unreached(),
        }
    }

    pub fn into_external_response(self) -> (resp: T)
    {
        match self {
            Response::ExternalResponse(resp) => resp,
            _ => unreached(),
        }
    }

    pub fn is_k_response(&self) -> (res: bool)
    {
        match self {
            Response::KResponse(_) => true,
            _ => false,
        }
    }

    pub fn as_k_response_ref(&self) -> (resp: &KubeAPIResponse)
    {
        match self {
            Response::KResponse(resp) => resp,
            _ => unreached(),
        }
    }

    pub fn into_k_response(self) -> (resp: KubeAPIResponse)
    {
        match self {
            Response::KResponse(resp) => resp,
            _ => unreached(),
        }
    }
}

impl <T: View> View for Request<T> {
    type V = (Option<KubeAPIRequestView>, Option<T::V>);

    open spec fn view(&self) -> (Option<KubeAPIRequestView>, Option<T::V>) {
        match self {
            Request::KRequest(r) => (Some(r.view()), None),
            Request::ExternalRequest(e) => (None, Some(e.view())),
        }
    }
}

#[macro_export]
macro_rules! is_some_k_get_resp {
    ($r:expr) => {
        $r.is_some() && $r.as_ref().unwrap().is_k_response()
        && $r.as_ref().unwrap().as_k_response_ref().is_get_response()
    };
}

#[macro_export]
macro_rules! is_some_k_create_resp {
    ($r:expr) => {
        $r.is_some() && $r.as_ref().unwrap().is_k_response()
        && $r.as_ref().unwrap().as_k_response_ref().is_create_response()
    };
}

#[macro_export]
macro_rules! is_some_k_update_resp {
    ($r:expr) => {
        $r.is_some() && $r.as_ref().unwrap().is_k_response()
        && $r.as_ref().unwrap().as_k_response_ref().is_update_response()
    };
}

#[macro_export]
macro_rules! extract_some_k_get_resp {
    ($r:expr) => {
        $r.unwrap().into_k_response().into_get_response().res
    };
}

#[macro_export]
macro_rules! extract_some_k_create_resp {
    ($r:expr) => {
        $r.unwrap().into_k_response().into_create_response().res
    };
}

#[macro_export]
macro_rules! extract_some_k_update_resp {
    ($r:expr) => {
        $r.unwrap().into_k_response().into_update_response().res
    };
}

pub use is_some_k_get_resp;
pub use is_some_k_create_resp;
pub use is_some_k_update_resp;
pub use extract_some_k_get_resp;
pub use extract_some_k_create_resp;
pub use extract_some_k_update_resp;

}

// Step 2 (view_refinement) VEval Score: Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 19
// Verified: -1, Errors: 999, Verus Errors: 19