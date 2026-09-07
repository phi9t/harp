import CrouzeixTextbook.Correspondence
import CrouzeixTextbook.ExportReceipt
import CrouzeixTextbook.HarpStatementAuditTests
import CrouzeixTextbook.HarpRemainderTests
import CrouzeixTextbook.HarpRemainderApplicationTests

unsafe def main (args : List String) : IO UInt32 :=
  CrouzeixTextbook.ExportReceipt.run args
