let tally limit =
  if limit < 0 then
    0
  else begin
    let accumulator = ref 0 in
    for cursor = 0 to limit do
      accumulator := !accumulator + cursor
    done;
    !accumulator
  end
