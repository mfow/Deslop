let aggregate bound =
  if bound < 0 then
    0
  else begin
    let running = ref 0 in
    for step = 0 to bound do
      running := !running + step;
      running := !running + 2
    done;
    !running
  end
