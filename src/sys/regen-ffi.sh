#!/bin/sh

BINDGEN=$1
NACL_DIR=$NACL_SDK_ROOT
NACL_INCLUDE_DIR=$NACL_DIR/include

OUT=lib.rs

rm -f $OUT
echo "#![allow(non_camel_case_types)]" >> $OUT
echo "#![allow(non_snake_case)]"       >> $OUT
echo "#![allow(raw_pointer_derive)]"   >> $OUT
echo "#![allow(missing_copy_implementations)]" >> $OUT
echo "extern crate libc;" >> $OUT

($NACL_DIR/toolchain/`$NACL_DIR/tools/getos.py`_pnacl/bin/pnacl-clang -std=gnu11 -dM -E -x c - < /dev/null) > builtin_defines.h

export LD_LIBRARY_PATH=$BINDGEN_DIR:$LD_LIBRARY_PATH

$BINDGEN -nostdinc -I $NACL_INCLUDE_DIR -isystem $NACL_DIR/toolchain/`$NACL_DIR/tools/getos.py`_pnacl/le32-nacl/include -I $NACL_DIR/toolchain/`$NACL_DIR/tools/getos.py`_pnacl/le32-nacl/usr/include -isystem $NACL_DIR/toolchain/`$NACL_DIR/tools/getos.py`_pnacl/le32-nacl/include/c++/v1/ -isystem $NACL_DIR/include/pnacl -isystem $NACL_DIR/toolchain/`$NACL_DIR/tools/getos.py`_pnacl/lib/clang/3.7.0/include -target le32-unknown-nacl ffi.h -pthread -o temp -D__BINDGEN__ -std=gnu11

cat temp >> $OUT
rm temp
