pub mod config;
pub mod errors;
pub mod logger;

// Macro stuff:
// $(), == repeating field
// $(),* == repeating field, separated by commas
// $(,)? == optional trailing comma

// It might be worthwhile to keep all the derives, even if it is boilerplatey...
// macro_rules! model {
//     (
//         $(#[$attr:meta])*
//         pub struct $name:ident {
//             $( $vis:vis $field:ident: $t:ty, )* $(,)?
//         }

//     ) => {
//         $(#[$attr])*
//         #[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
//         pub struct $name {
//             $( pub $field: $t ),*
//         }

//     };

//     (
//         $(#[$attr:meta])*
//         pub enum $name:ident {
//             $( $field:ident, )* $(,)?
//         }

//     ) => {
//         $(#[$attr])*
//         #[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
//         pub enum $name {
//             $( $field ),*
//         }

//     };
// }
