#[cfg(all(feature="backend_cairo", feature="backend_vello"))]
compile_error!("Only one of the features 'backend_cairo' and 'backend_vello' can be enabled at a time");

#[cfg(all(not(feature="backend_cairo"), not(feature="backend_vello")))]
compile_error!("One of the features 'backend_cairo' and 'backend_vello' must be enabled");

#[cfg(feature="backend_cairo")]
pub mod cairo;
#[cfg(feature="backend_vello")]
pub mod vello;

pub trait Composable {
    type Config;
    type Return;

    fn compose(config: Self::Config) -> Self::Return;
}