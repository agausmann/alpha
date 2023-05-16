#include <inttypes.h>
#include <stdio.h>

struct PieceSet {
  int32_t kings;
  int32_t queens;
  int32_t rooks;
  int32_t bishops;
  int32_t knights;
  int32_t pawns;
};

struct PieceSet bijele(struct PieceSet pieces) {
  struct PieceSet difference;
  difference.kings = 1 - pieces.kings;
  difference.queens = 1 - pieces.queens;
  difference.rooks = 2 - pieces.rooks;
  difference.bishops = 2 - pieces.bishops;
  difference.knights = 2 - pieces.knights;
  difference.pawns = 8 - pieces.pawns;
  return difference;
}

int main() {
  struct PieceSet pieces;
  scanf(
    "%d %d %d %d %d %d",
    &pieces.kings,
    &pieces.queens,
    &pieces.rooks,
    &pieces.bishops,
    &pieces.knights,
    &pieces.pawns
  );
  struct PieceSet difference = bijele(pieces);
  printf(
    "%d %d %d %d %d %d\n",
    difference.kings,
    difference.queens,
    difference.rooks,
    difference.bishops,
    difference.knights,
    difference.pawns
  );
  return 0;
}
