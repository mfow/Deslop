let describe code =
  if code = 200 then
    "ok"
  else if code = 404 then
    "missing"
  else if code = 500 then
    "error"
  else
    "unknown"
