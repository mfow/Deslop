let both_ready alpha beta gamma delta =
  let first = alpha && beta in
  let second = gamma && delta in
  let third = first && second in
  third && alpha
