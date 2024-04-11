# telety

<!-- cargo-rdme start -->

Access type information across crates and modules in your proc macros

## Creating telety information
Simply apply the attribute to a supported item and provide the current module path as an arguments:
```rust
pub mod my_mod {
    #[telety(crate::my_mod)]
    pub struct MyStruct;
}
```
If the item has other attributes, `#[telety]` should be placed after the last attribute which modifies the item definition.
## Using telety information
The v* (e.g. `v0`, `v1`) modules contain objects for generating the TokenStreams to read telety information.  
You will need two macros (or one that has two modes), one to generate the code to read the information,
and a second to use the information for your own purposes.  
The process is a bit cumbersome, but works like this:
1. Your proc macro calls `Command::apply` with the path to a macro and returns the output.
2. The generated tokens will invoke the `#[telety]`-generated macro for the type (or a fallback).
3. The `#[telety]`-generated macro will textually insert the information where requested,
   usually as arguments to an invocation of your second macro.
4. Your second proc macro then can use the requested information.
    1. If this information was the definition of the item, you can create a `Telety` object.
    2. With `Telety::alias_of`, you can access aliases to any type referenced in the item. These aliases have
       global paths, so they can be used in other contexts.
    3. If the item is generic, you can use `Telety::generics_visitor` to substitute generic arguments into the alias.
#### Example
Here's how we could write `mix!`, a proc macro which combines the fields of two structs into a new struct.
Two types from different crates that we want to combine:
```rust
#[telety(crate)]
pub struct Water {
    pub water_liters: f32,
    pub source: water::Source,
}
```
```rust
#[telety(crate)]
pub struct Oil {
    pub oil_liters: f32,
    pub variety: oil::Variety,
}
```
We define our first macro which takes paths to structs:
```rust

/// mix!(path_to_struct0, path_to_struct1, new_struct_ident);
#[proc_macro]
pub fn mix(tokens: TokenStream) -> TokenStream {
    // Split `tokens` to `path_to_struct0`, `path_to_struct1`, & `new_struct_ident`
    // ...
    // Take the relative paths `path_to_struct0` and `path_to_struct1`
    // and use v1::TY::apply to call mix_impl! with the actual definition
    let item0: syn::Path = parse2(path_to_struct0)?;
    let item1: syn::Path = parse2(path_to_struct1)?;
    
    // telety works by find and replace - define a 'needle', and put it
    // where you want the type information inserted.
    let needle0: syn::Ident = parse_quote!(item0_goes_here);
    let needle1: syn::Ident = parse_quote!(item1_goes_here);
    // This macro generates the call to our actual implementation.
    // The `TY.apply` calls will replace the needles with the type definitions.
    let output = quote! {
        ::my_crate::mix_impl!(#needle0, #needle1, #new_struct_ident);
    };
    
    let output = telety::v1::TY.apply(
        item0,
        needle0,
        output,
    );
    let output = telety::v1::TY.apply(
        item1,
        needle1,
        output.into_token_stream(),
    );
    output
}
```
The first macro will generate a call to our second macro with the definitions of the two structs.
```rust
/// mix_impl!(struct0_definition, struct1_definition, new_struct_ident);
#[proc_macro]
pub fn mix_impl(tokens: TokenStream) -> TokenStream {
    // Parse macro arguments
    // ...
    let item0: syn::Item = parse2(struct0_definition)?;
    let item1: syn::Item = parse2(struct1_definition)?;
    // Telety lets us reference remote types
    let telety0 = Telety::new(&item0);
    let telety1 = Telety::new(&item1);
    // Get the fields from the struct definitions
    // ...
    // Change the original type tokens to our aliases
    for field in fields0.iter_mut() {
        // We can get a location-independent alias for any type
        // used in the original item definition.
        let mut aliased_ty = telety0.alias_of(&field.ty).unwrap();
        // Switch to `crate::...` if in the same crate the alias was defined,
        // otherwise keep the path as `::my_crate::...`.
        telety::visitor::Crateify::new().visit_type_mut(&mut aliased_ty);
        field.ty = aliased_ty;
    }
    for field in fields1.iter_mut() {
        let mut aliased_ty = telety1.alias_of(&field.ty).unwrap();
        telety::visitor::Crateify::new().visit_type_mut(&mut aliased_ty);
        field.ty = aliased_ty;
    }

    // Create a new struct with all the fields from both mixed types
    quote::quote! {
        pub struct #new_struct_ident {
            #fields0
            #fields1
        }
    }
}
```
## Limitations
* telety is not yet robust in handling all features of types.
  Expect failures if your types have lifetimes, const generics, associated types, impl types, or dyn types.
* Items cannot currently contain types which are less public than them. e.g.
  ```rust
  struct Private;
  #[telety(crate)]
  pub struct Public(Private);
  ```
  will not compile.
* You cannot have a macro with the same name as the item in the same module, as telety needs to define its own.
* Type aliases (e.g. `type MyAlias = MyType`) do not propagate the macro, so any telety information cannot be accessed through the alias. (In the future this should be possible by adding an attribute.)
* Importing using the `use my_mod::MyStruct::{self}` syntax only imports the type, and not macros or values. Telety information will not be imported.
## How it works
The `#[telety(...)]` attribute scans the item for all distinct referenced types.
It then creates a hidden module which contains a type alias for each type.
If there exists a macro in the same path as the original type, the macro is aliased as well.  
Now, if you have the canonical path to the type, you can locate the module of aliases from anywhere.
If you also have the original item definition, you can map a `syn::Type` to an alias which works anywhere.  
A macro is created with the same name as the item. The macro contains information about the associated type,
including the canonical path and the item defininion.  
Because the macro and the item have the same name, but exist in separate namespaces, the macro can,
in most circumstances, be located by using the same tokens as the type.  
For example:
```rust
use my_crate::my_module;

// my_module::MyType is a type with the #[telety(...)] macro
struct NewType(my_module::MyType);
// The telety-created macro can also be used with the same path
my_module::MyType!(...);
```
`Command::apply` generates code to utilize this macro. If you are unsure whether a type is `#[telety]`,
`Apply::with_fallback` utilizes special language techniques to allow for fallback behavior,
instead of resulting in a compile error (subject to some limitations, be sure to review the documentation).

<!-- cargo-rdme end -->
