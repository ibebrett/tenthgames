#include "raylib.h"

int main() {
  int t = 0;
  InitWindow(1200, 700, "Hello");
  SetTargetFPS(60);

  // Texture filter mode for pixel art.

  while (!WindowShouldClose()) {
    t += 1;

    BeginDrawing();
    ClearBackground(RAYWHITE);

    DrawText("Hello!", 100, 100, 20, LIGHTGRAY);
  
    EndDrawing();
  }
}
