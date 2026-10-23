open Stdlib
open Printf

let total_eligible values =
  let total = ref 0 in
  List.iter
    (fun value ->
      if value > 0 then
        let adjusted = value * 3 in
        total := !total + adjusted
      else
        total := !total + 1)
    values;
  !total
