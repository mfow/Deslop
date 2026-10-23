let tally words =
  let counts = Hashtbl.create 16 in
  List.iter
    (fun word ->
      let current = Option.value (Hashtbl.find_opt counts word) ~default:0 in
      Hashtbl.replace counts word (current + 1))
    words;
  counts
