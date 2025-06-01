#![allow(unused_imports)]
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

pub enum Request<T: View> {
    KRequest(KubeAPIRequest),
    ExternalRequest(T),
}

#[is_variant]
pub enum Response<T: View> {
    KResponse(KubeAPIResponse),
    ExternalResponse(T),
}

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
        ensures
            res <==> self.view().1.is_Some(),
    {
        match self {
            Response::ExternalResponse(_) => true,
            _ => false,
        }
    }

    pub fn as_external_response_ref(&self) -> (resp: &T)
        requires
            self.is_external_response(),
        ensures
            self.view().1 == Some(resp@),
    {
        match self {
            Response::ExternalResponse(resp) => resp,
            _ => unreached(),
        }
    }

    pub fn into_external_response(self) -> (resp: T)
        requires
            self.is_external_response(),
        ensures
            self.view().1 == Some(resp@),
    {
        match self {
            Response::ExternalResponse(resp) => resp,
            _ => unreached(),
        }
    }

    pub fn is_k_response(&self) -> (res: bool)
        ensures
            res <==> self.view().0.is_Some(),
    {
        match self {
            Response::KResponse(_) => true,
            _ => false,
        }
    }

    pub fn as_k_response_ref(&self) -> (resp: &KubeAPIResponse)
        requires
            self.is_k_response(),
        ensures
            self.view().0 == Some(resp.view()),
    {
        match self {
            Response::KResponse(resp) => resp,
            _ => unreached(),
        }
    }

    pub fn into_k_response(self) -> (resp: KubeAPIResponse)
        requires
            self.is_k_response(),
        ensures
            self.view().0 == Some(resp.view()),
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

// -------------------- Macros --------------------

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

// -------------------- Final inv definitions --------------------

impl<T: View> Request<T> {
    pub closed spec fn inv(&self) -> bool {
        match self {
            Request::KRequest(_) => true,
            Request::ExternalRequest(_) => true,
        }
    }
}

impl<T: View> Response<T> {
    pub closed spec fn inv(&self) -> bool {
        match self {
            Response::KResponse(_) => true,
            Response::ExternalResponse(_) => true,
        }
    }
}

}

// Repair Round 7 VEval Score: Compilation Error: True, Verified: -1, Errors: 999, Verus Errors: 10
// Verified: -1, Errors: 999, Verus Errors: 10