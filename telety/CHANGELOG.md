# telety

## 0.3.0
* alias::Map can now create 'sub-maps' containing additional aliases which aren't parsed by telety (such as those appearing in attributes).
* Added the 'proxy' argument to the telety attribute. This can be used to suppliment telety information to a third-party item.
  ``` rust
  #[telety(crate, proxy = "std::fmt::Debug")]
  pub trait Debug {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error>;
  }
  ```
  The above imports the std::fmt::Debug trait along with a generated telety macro using the attributed definition (the definition is only used in the macro, it is otherwise discarded from the attribute output).
  Note that all types in the definition must be in scope.
* Converted syn visitors to directed-visit visitors. This allows them to be used with custom AST extensions, such as attribute meta contents.
* Hide more warnings from generated macros.

## 0.2.0
* Added telety_path parameter to #[telety] to specify the location of the telety crate contents
* Commands changed to a builder API which takes plain TokenStreams instead of already defined macros for fallback behavior.
* Removed some implementation details from the public API
* Moved some code from telety-impl to telety-macro which was not needed by telety

## 0.1.0
* Initial release