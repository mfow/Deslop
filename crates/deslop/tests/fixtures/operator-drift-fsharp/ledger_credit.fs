module LedgerCredit

let postEntries entries rate floor ceiling =
    let mutable running = 0.0
    let adjusted = ResizeArray<float>()
    for entry in entries do
        let scaled = entry * rate
        let shifted = scaled + floor
        let clamped = min shifted ceiling
        adjusted.Add clamped
        running <- running + clamped
    let average = running / float (max adjusted.Count 1)
    let spread = Seq.max adjusted - Seq.min adjusted
    (running, average, spread)
