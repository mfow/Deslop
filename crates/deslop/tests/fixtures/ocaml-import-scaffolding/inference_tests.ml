open Alcotest
open Xml_schema
open Data
open Data.Runtime
open Data.Runtime.Base_types
open Data.Runtime.Structural_types
open Provider_implementation

let describe_schema name =
  if String.length name > 0 then name else "unknown"
