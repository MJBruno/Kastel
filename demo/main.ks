from std.io import {
    buf_reader,
    buf_writer,
    SeekFrom
}

let reader = buf_reader("journal.txt")
let line = reader.read_line()
reader.seek(0, SeekFrom.Start)
reader.close()

let writer = buf_writer("rapport.txt")
writer.write_all("Rapport Kastel\n")
writer.flush()
writer.close()
