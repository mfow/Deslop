let render_ledger rows sink =
  List.iter
    (fun (label, amount) ->
      Buffer.add_string sink label;
      Buffer.add_string sink (string_of_int amount))
    rows;
  Buffer.contents sink
