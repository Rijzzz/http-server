use std::collections::HashMap;
use std::sync::Arc;

use crate::{Method, Request, Response};

pub type Handler = Arc<dyn Fn(&Request, &HashMap<String, String>) -> Response + Send + Sync>;
pub type Middleware = Arc<dyn Fn(&Request) -> Option<Response> + Send + Sync>;

struct Route {
    method: Method,
    pattern: Vec<PatternSegment>,
    handler: Handler,
}

enum PatternSegment {
    Exact(String),
    Param(String),
}

#[derive(Default)]
pub struct Router {
    routes: Vec<Route>,
    middlewares: Vec<Middleware>,
}

pub enum RouteMatch<'a> {
    Found(&'a Handler, HashMap<String, String>),
    MethodNotAllowed(Vec<&'static str>),
    NotFound,
}

impl Router {
    #[must_use]
    pub fn new() -> Self {
        Self {
            routes: Vec::new(),
            middlewares: Vec::new(),
        }
    }

    pub fn use_middleware<F>(&mut self, f: F)
    where
        F: Fn(&Request) -> Option<Response> + Send + Sync + 'static,
    {
        self.middlewares.push(Arc::new(f));
    }

    pub fn add_route<F>(&mut self, method: Method, path: &str, handler: F)
    where
        F: Fn(&Request, &HashMap<String, String>) -> Response + Send + Sync + 'static,
    {
        let segments = parse_pattern(path);
        self.routes.push(Route {
            method,
            pattern: segments,
            handler: Arc::new(handler),
        });
    }

    pub fn get<F>(&mut self, path: &str, handler: F)
    where
        F: Fn(&Request, &HashMap<String, String>) -> Response + Send + Sync + 'static,
    {
        self.add_route(Method::Get, path, handler);
    }

    pub fn post<F>(&mut self, path: &str, handler: F)
    where
        F: Fn(&Request, &HashMap<String, String>) -> Response + Send + Sync + 'static,
    {
        self.add_route(Method::Post, path, handler);
    }

    pub fn put<F>(&mut self, path: &str, handler: F)
    where
        F: Fn(&Request, &HashMap<String, String>) -> Response + Send + Sync + 'static,
    {
        self.add_route(Method::Put, path, handler);
    }

    pub fn delete<F>(&mut self, path: &str, handler: F)
    where
        F: Fn(&Request, &HashMap<String, String>) -> Response + Send + Sync + 'static,
    {
        self.add_route(Method::Delete, path, handler);
    }

    pub fn patch<F>(&mut self, path: &str, handler: F)
    where
        F: Fn(&Request, &HashMap<String, String>) -> Response + Send + Sync + 'static,
    {
        self.add_route(Method::Patch, path, handler);
    }

    #[must_use]
    pub fn handle(&self, req: &Request) -> Response {
        for mw in &self.middlewares {
            if let Some(res) = mw(req) {
                return res;
            }
        }

        match self.find_match(req.method, &req.path) {
            RouteMatch::Found(handler, params) => handler(req, &params),
            RouteMatch::MethodNotAllowed(allowed) => Response::method_not_allowed(&allowed),
            RouteMatch::NotFound => Response::not_found(),
        }
    }

    #[must_use]
    pub fn find_match<'a>(&'a self, method: Method, path: &str) -> RouteMatch<'a> {
        let req_segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        let mut matched_methods = Vec::new();
        let mut exact_handler = None;
        let mut exact_params = HashMap::new();

        for route in &self.routes {
            if segments_match(&route.pattern, &req_segments) {
                matched_methods.push(route.method.as_str());
                if route.method == method && exact_handler.is_none() {
                    exact_handler = Some(&route.handler);
                    exact_params = extract_params(&route.pattern, &req_segments);
                }
            }
        }

        if let Some(h) = exact_handler {
            RouteMatch::Found(h, exact_params)
        } else if !matched_methods.is_empty() {
            RouteMatch::MethodNotAllowed(matched_methods)
        } else {
            RouteMatch::NotFound
        }
    }
}

fn parse_pattern(path: &str) -> Vec<PatternSegment> {
    path.split('/')
        .filter(|s| !s.is_empty())
        .map(|seg| {
            if let Some(param) = seg.strip_prefix(':') {
                PatternSegment::Param(param.to_string())
            } else {
                PatternSegment::Exact(seg.to_string())
            }
        })
        .collect()
}

fn segments_match(pattern: &[PatternSegment], req_segments: &[&str]) -> bool {
    if pattern.len() != req_segments.len() {
        return false;
    }
    for (pat, req) in pattern.iter().zip(req_segments.iter()) {
        match pat {
            PatternSegment::Exact(s) => {
                if s != req {
                    return false;
                }
            }
            PatternSegment::Param(_) => {}
        }
    }
    true
}

fn extract_params(pattern: &[PatternSegment], req_segments: &[&str]) -> HashMap<String, String> {
    let mut params = HashMap::new();
    for (pat, req) in pattern.iter().zip(req_segments.iter()) {
        if let PatternSegment::Param(name) = pat {
            params.insert(name.clone(), (*req).to_string());
        }
    }
    params
}
