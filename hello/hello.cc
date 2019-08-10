#include "raylib.h"

int main() {
  int t = 0;
  InitWindow(1200, 700, "Hello");
  
  Vector2 ballPosition = { (float)1000, (float)100 };
  SetTargetFPS(60);

 
  // Texture filter mode for pixel art.

  while (!WindowShouldClose()) {
    t += 1;

    if (IsKeyDown(KEY_RIGHT)) ballPosition.x += 2.0f;
    if (IsKeyDown(KEY_LEFT)) ballPosition.x -= 2.0f;
    if (IsKeyDown(KEY_UP)) ballPosition.y -= 2.0f;
    if (IsKeyDown(KEY_DOWN)) ballPosition.y += 2.0f;
         

    BeginDrawing();
    ClearBackground(RAYWHITE);
    
    DrawText("Hello!", 100, 100, 20, LIGHTGRAY);
    DrawText("Prepare",100, 150, 20, BLACK);
    DrawCircle(600,350,50.0,RED);
    DrawCircleLines(600,350,150.0,BLACK);

    DrawCircleV(ballPosition, 25, BLACK);
  
    EndDrawing();
  }
}
