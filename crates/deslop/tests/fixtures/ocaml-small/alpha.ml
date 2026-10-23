let accumulate limit =
  let total = ref 0 in
  for index = 1 to limit do
    if index mod 2 = 0 then
      total := !total + (index * 3)
    else
      total := !total + 1
  done;
  !total
