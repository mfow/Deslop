(* dropped block comment *)
(** dropped doc comment *)
open Printf

let probe count =
  let label = "text" in
  let ratio = 3.5 in
  let flag = true in
  let glyph = 'x' in
  let hex = 0xFF in
  let nothing = () in
  let quoted = {|raw|} in
  let shown = sprintf "%d" count in
  let found = Some label in
  if flag && count >= 0 then ratio *. float_of_int count else -.ratio
