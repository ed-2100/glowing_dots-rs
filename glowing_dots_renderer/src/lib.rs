mod allocator;
mod helpers;
mod instance;
mod loader;
mod surface;

pub use allocator::*;
pub use helpers::*;
pub use instance::*;
pub use loader::*;
pub use surface::*;

struct MyStruct<Features> {
    value: i32,
    features: Features
}

struct NoFeatures;

// No features enabled
impl MyStruct<NoFeatures> {
    fn new() -> Self {
        MyStruct {
            value: 0,
            features: NoFeatures,
        }
    }
}

struct FeatureA;

// Feature A enabled
impl<F> MyStruct<F> {
    fn with_feature_a(self) -> MyStruct<(FeatureA, F)> {
        MyStruct {
            value: self.value,
            features: (FeatureA, self.features),
        }
    }
}

trait HasFeatureA {
    fn do_something(&self);
}

impl<F> HasFeatureA for MyStruct<(FeatureA, F)> {
    fn do_something(&self) {
        println!("Feature A enabled: {}", self.value);
    }
}

struct FeatureB {
    data: i32
}

// Feature B enabled
impl<F> MyStruct<F> {
    fn with_feature_b(self) -> MyStruct<(FeatureB, F)> {
        MyStruct {
            value: self.value,
            features: (FeatureB {
                data: 9
            }, self.features),
        }
    }
}

trait HasFeatureB {
    fn do_something_else(&self);
}

impl<F> HasFeatureB for MyStruct<(FeatureB, F)> {
    fn do_something_else(&self) {
        println!("Feature B enabled: {}", self.features.0.data);
    }
}
