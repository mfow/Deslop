let combine bound =
  let sum = ref 0 in
  for step = 1 to bound do
    if step mod 2 = 0 then
      sum := !sum + (step * 7)
    else
      sum := !sum + 4
  done;
  !sum
