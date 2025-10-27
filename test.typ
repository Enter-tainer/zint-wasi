#import "typst-package/lib.typ": barcode

#set page(width: 300pt, height: auto, margin: 10pt)

#barcode("Hello, World!", "QRCode")

#barcode("12345678", "Code128")

#barcode("9780201379624", "EANX")
