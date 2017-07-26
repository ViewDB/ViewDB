(
  Copyright (c) 2017, All Contributors (see CONTRIBUTORS file)

  This Source Code Form is subject to the terms of the Mozilla Public
  License, v. 2.0. If a copy of the MPL was not distributed with this
  file, You can obtain one at http://mozilla.org/MPL/2.0/.
)

$ATTRVALPREFIX : 0x00.
$ATTRINDEXPREFIX : 0x01.

ATTRID : HASH/SHA1.

ATTR : (TODO: indexing: `3DUP ...`)
       (prepare attribute value pair)
       $ATTRVALPREFIX ROT ATTRID CONCAT ROT TXID CONCAT CONCAT SWAP.