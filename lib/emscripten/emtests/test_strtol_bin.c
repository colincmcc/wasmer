/*
 * Copyright 2016 The Emscripten Authors.  All rights reserved.
 * Emscripten is available under two separate licenses, the MIT license and the
 * University of Illinois/NCSA Open Source License.  Both these licenses can be
 * found in the LICENSE file.
 */

#include <stdio.h>
#include <stdlib.h>

int main() {
  const char *STRING = "1 -101 +1011";
  char *end_char;

  // defined base
  long l4 = strtol(STRING, &end_char, 2);
  long l5 = strtol(end_char, &end_char, 2);
  long l6 = strtol(end_char, NULL, 2);

  printf("%d%d%d\n", l4 == 1, l5 == -5, l6 == 11);
  return 0;
}
