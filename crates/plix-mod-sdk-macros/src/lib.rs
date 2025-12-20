//! Plix Mod SDK Procedural Macros
//!
//! This crate provides attribute macros for mod development:
//! - `#[plix_mod]` - Mark a struct/impl as the mod entry point
//! - `#[on_event("event_name")]` - Register an event handler
//!
//! # Example
//!
//! ```rust,ignore
//! use plix_mod_sdk::prelude::*;
//! use plix_mod_sdk_macros::{plix_mod, on_event};
//!
//! #[plix_mod]
//! struct MyChatFilter;
//!
//! #[plix_mod]
//! impl MyChatFilter {
//!     fn init(&self) {
//!         info!("Chat filter initialized!");
//!         subscribe(EventType::PlayerChat).unwrap();
//!     }
//!
//!     fn shutdown(&self) {
//!         info!("Chat filter shutting down");
//!     }
//!
//!     #[on_event("on_player_chat")]
//!     fn handle_chat(&self, ctx: &EventContext, payload: PlayerChatPayload) {
//!         if payload.text.contains("badword") {
//!             ctx.cancel().unwrap();
//!         }
//!     }
//! }
//! ```

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, ItemImpl, LitStr};

/// Mark a struct as the mod entry point
///
/// When applied to a struct, this registers it as the mod type.
/// When applied to an impl block, this generates the required WASM exports:
/// - `mod_init` - Called when mod is loaded
/// - `mod_on_event` - Called when an event occurs
/// - `mod_shutdown` - Called when mod is unloaded
///
/// # Example
///
/// ```rust,ignore
/// #[plix_mod]
/// struct MyMod;
///
/// #[plix_mod]
/// impl MyMod {
///     fn init(&self) {
///         // Called on mod_init
///     }
///
///     fn shutdown(&self) {
///         // Called on mod_shutdown
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn plix_mod(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Try to parse as struct first
    if let Ok(input) = syn::parse::<DeriveInput>(item.clone()) {
        return generate_struct_code(input);
    }

    // Try to parse as impl block
    if let Ok(input) = syn::parse::<ItemImpl>(item.clone()) {
        return generate_impl_code(input);
    }

    // Return original item with compile error
    let item2: proc_macro2::TokenStream = item.into();
    quote! {
        compile_error!("#[plix_mod] can only be applied to a struct or impl block");
        #item2
    }
    .into()
}

/// Generate code for struct declaration
fn generate_struct_code(input: DeriveInput) -> TokenStream {
    let name = &input.ident;
    let vis = &input.vis;
    let attrs = &input.attrs;

    quote! {
        #(#attrs)*
        #vis struct #name;

        // Static instance
        static mut __PLIX_MOD_INSTANCE: Option<#name> = None;

        // Helper to get mod instance
        #[allow(dead_code)]
        fn __plix_get_mod() -> &'static #name {
            unsafe {
                __PLIX_MOD_INSTANCE.get_or_insert_with(|| #name)
            }
        }
    }
    .into()
}

/// Generate code for impl block with exports
fn generate_impl_code(input: ItemImpl) -> TokenStream {
    let self_ty = &input.self_ty;
    let items = &input.items;

    // Collect event handlers
    let mut event_handlers: Vec<(String, syn::Ident)> = Vec::new();

    for item in items {
        if let syn::ImplItem::Fn(method) = item {
            for attr in &method.attrs {
                if attr.path().is_ident("on_event") {
                    if let Ok(event_name) = attr.parse_args::<LitStr>() {
                        event_handlers.push((event_name.value(), method.sig.ident.clone()));
                    }
                }
            }
        }
    }

    // Generate event dispatch match arms
    let event_dispatch = event_handlers.iter().map(|(event_name, method_name)| {
        let event_id = event_name_to_id(event_name);
        quote! {
            #event_id => {
                // TODO: Decode payload and call handler
                let _ = instance.#method_name();
                0
            }
        }
    });

    quote! {
        impl #self_ty {
            #(#items)*
        }

        // WASM exports
        #[no_mangle]
        pub extern "C" fn mod_init() -> i32 {
            let instance = __plix_get_mod();
            if let Some(init_fn) = std::panic::catch_unwind(|| {
                instance.init();
            }).ok() {
                0
            } else {
                -1
            }
        }

        #[no_mangle]
        pub extern "C" fn mod_on_event(event_type: i32, _payload_ptr: i32, _payload_len: i32) -> i32 {
            let instance = __plix_get_mod();
            match event_type as u8 {
                #(#event_dispatch)*
                _ => 0
            }
        }

        #[no_mangle]
        pub extern "C" fn mod_shutdown() -> i32 {
            let instance = __plix_get_mod();
            if let Some(_) = std::panic::catch_unwind(|| {
                instance.shutdown();
            }).ok() {
                0
            } else {
                -1
            }
        }
    }
    .into()
}

/// Convert event name to event ID
fn event_name_to_id(name: &str) -> u8 {
    match name {
        "on_server_start" | "server_start" => 0x01,
        "on_server_stop" | "server_stop" => 0x02,
        "on_player_join" | "player_join" => 0x03,
        "on_player_leave" | "player_leave" => 0x04,
        "on_player_chat" | "player_chat" => 0x05,
        "on_block_placed" | "block_placed" => 0x06,
        "on_block_broken" | "block_broken" => 0x07,
        "on_entity_spawned" | "entity_spawned" => 0x08,
        "on_entity_despawned" | "entity_despawned" => 0x09,
        _ => 0xFF, // Unknown event
    }
}

/// Register an event handler
///
/// Applied to methods in a `#[plix_mod]` impl block to register them
/// as handlers for specific events.
///
/// # Example
///
/// ```rust,ignore
/// #[on_event("on_player_chat")]
/// fn handle_chat(&self, ctx: &EventContext, payload: PlayerChatPayload) {
///     // Handle chat event
/// }
/// ```
#[proc_macro_attribute]
pub fn on_event(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse event name from attribute
    let event_name = parse_macro_input!(attr as LitStr);
    let _event_id = event_name_to_id(&event_name.value());

    // For now, just pass through the item
    // The actual registration happens in #[plix_mod] on the impl block
    item
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_name_to_id() {
        assert_eq!(event_name_to_id("on_player_chat"), 0x05);
        assert_eq!(event_name_to_id("player_chat"), 0x05);
        assert_eq!(event_name_to_id("on_block_placed"), 0x06);
        assert_eq!(event_name_to_id("unknown"), 0xFF);
    }
}
