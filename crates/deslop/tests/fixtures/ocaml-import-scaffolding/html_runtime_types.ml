open Alcotest
open Ounit
open Stdlib
open Printf
open Data
open Data.Runtime
open Data.Runtime.Base_types

let build_html name =
  sprintf "<main>%s</main>" name
