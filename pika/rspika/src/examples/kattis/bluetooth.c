#include <inttypes.h>
#include <stdio.h>
#include <stdbool.h>
#include <ctype.h>

enum Tooth {
  Healthy,
  Blue,
  Missing,
};

int32_t bluetooth(enum Tooth teeth[2][2][8]) {
  bool blue[2] = {false, false};
  bool jaw[2][2] = {{false, false}, {false, false}};
  for (int i = 0; i < 2; i++) {
    for (int j = 0; j < 2; j++) {
      for (int k = 0; k < 8; k++) {
        if (teeth[i][j][k] == Blue) {
          //printf("blue %d %d %d\n", i, j, k);
          blue[i] = true;
        } else if (teeth[i][j][k] == Healthy) {
          //printf("healthy %d %d %d\n", i, j, k);
          jaw[i][j] = true;
        }
      }
    }
  }
  for (int i = 0; i < 2; i++) {
    //printf("%d %d %d\n", blue[i], jaw[i][0], jaw[i][1]);
    if (!blue[i] && jaw[i][0] && jaw[i][1]) {
      return i;
    }
  }
  return 2;
}

int main() {
  enum Tooth teeth[2][2][8];
  for (int i = 0; i < 2; i++) {
    for (int j = 0; j < 2; j++) {
      for (int k = 0; k < 8; k++) {
        teeth[i][j][k] = Healthy;
      }
    }
  }

  int n;
  scanf("%d", &n);
  char c;
  for (int in = 0; in < n; in++) {

    // Get tooth index:
    c = getchar();
    while (isspace(c)) {
      c = getchar();
    }
    char a = getchar();

    int i = 0;
    if (isdigit(c)) {
      // Right side
      i = 1;

      // Normalize 8- to -8 by swapping a,c
      char tmp = a;
      a = c;
      c = tmp;
    }

    int j = 0;
    if (c == '+') {
      // Upper jaw
      j = 1;
    }

    int k = a - '0' - 1;

    //printf("%c %c\n", a, c);

    // Get tooth state:
    c = getchar();
    while (isspace(c)) {
      c = getchar();
    }

    //printf("%d %d %d %c\n", i, j, k, c);

    if (c == 'm') {
      teeth[i][j][k] = Missing;
    } else if (c == 'b') {
      teeth[i][j][k] = Blue;
    } else {
      //printf("unknown %c\n", c);
    }
  }

  int32_t result = bluetooth(teeth);
  printf("%d\n", result);
  return 0;
}
