open Stdlib
open Printf
open Provider_implementation
open Provider_implementation.Provided_types
open Provider_implementation.Provider_helpers
open Data
open Data.Runtime
open Data.Runtime.Base_types
open Data.Runtime.Structural_types
open Data.Runtime.Structural_inference
open Net

[@@@warning "-33"]

let render_json value =
  match value with "" -> "empty" | text -> String.trim text
