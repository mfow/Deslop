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
open Xml_schema
open Xml_linq
open Net

[@@@warning "-33"]

let render_xml value =
  String.split_on_char ':' value |> List.length
