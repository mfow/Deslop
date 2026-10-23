let post_entries entries rate floor ceiling =
  let running = ref 0.0 in
  let adjusted = ref [] in
  List.iter
    (fun entry ->
      let scaled = entry *. rate in
      let shifted = scaled -. floor in
      let clamped = Float.min shifted ceiling in
      adjusted := clamped :: !adjusted;
      running := !running +. clamped)
    entries;
  let average = !running /. float_of_int (max (List.length !adjusted) 1) in
  let spread = List.fold_left Float.max 0.0 !adjusted -. List.fold_left Float.min 0.0 !adjusted in
  (!running, average, spread)
