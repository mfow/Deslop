module Control

let renderLedger (rows: seq<string * int>) (sink: System.IO.TextWriter) =
    for (label, amount) in rows do
        sink.Write label
        sink.Write amount
    sink.Flush()
    sink
