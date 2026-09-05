import CrouzeixTextbook.Correspondence
import CrouzeixTextbook.ExportReceipt

unsafe def main (args : List String) : IO UInt32 :=
  CrouzeixTextbook.ExportReceipt.run args
