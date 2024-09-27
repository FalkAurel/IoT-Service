pub struct ServiceManager {
    services: Vec<dyn Layer>
}

// Middleware service, if it completes it can pass it to another service
pub trait Layer<S> {
    type Service;
    fn execute_service(&self, inner: S) -> Option<Self::Service>;
}

impl <T: ?Sized + Layer<S>, S> Layer<S> for T {
    type Service = T::Service;

    fn execute_service(&self, inner: S) -> Option<Self::Service> {
        self.execute_service(inner)
    }
}
