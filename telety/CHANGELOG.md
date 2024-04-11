# telety

## 0.2.0
* Added telety_path parameter to #[telety] to specify the location of the telety crate contents
* Commands changed to a builder API which takes plain TokenStreams instead of already defined macros for fallback behavior.
* Removed some implementation details from the public API
* Moved some code from telety-impl to telety-macro which was not needed by telety

## 0.1.0
* Initial release