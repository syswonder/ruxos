/*
 * 2048 clone in C
 *
 * Author: Dave Eddy <dave@daveeddy.com>
 * Date: 3/25/14
 * License: MIT
 */

/**
 * Edited by: AuYang261 <xu_jyang@163.com>
 * Date: 17/12/23
 * Link: https://github.com/bahamas10/2048.c
 */

#include <stdio.h>  // printf
#include <stdlib.h> // srand, rand
#include <string.h> // strlen
#include <time.h>   // time
#include <unistd.h> // usleep, sleep, getopt

// config
#define BOARD_SIZE 4

// clear the terminal
#define screen_clear()                \
    do {                              \
        printf("%s", "\033[H\033[J"); \
    } while (0)

// colors
#define ANSI_COLOR_RESET      "\x1b[0m"
#define ANSI_COLOR_BOLD       "\x1b[1m"
#define ANSI_COLOR_REVERSE    "\x1b[7m"
#define ANSI_COLOR_BG_RED     "\x1b[41m"
#define ANSI_COLOR_BG_GREEN   "\x1b[42m"
#define ANSI_COLOR_BG_YELLOW  "\x1b[43m"
#define ANSI_COLOR_BG_BLUE    "\x1b[44m"
#define ANSI_COLOR_BG_MAGENTA "\x1b[45m"
#define ANSI_COLOR_BG_CYAN    "\x1b[46m"
#define ANSI_COLOR_BG_WHITE   "\x1b[47m"

// directions
enum { DIRECTION_UP = 0, DIRECTION_DOWN, DIRECTION_LEFT, DIRECTION_RIGHT };

// number of moves
int moves = 0;

// options
int animations = 0;
int use_colors = 1;
int goal = 2048;

// the gameboard
int board[BOARD_SIZE * BOARD_SIZE];

// copy of the gameboard for use when determining if a move results
// in changes to the board
int _board[BOARD_SIZE * BOARD_SIZE];

// prototypes
int main(int argc, char **argv);
void initialize();
int index2d(int x, int y);
int index1dX(int index);
int index1dY(int index);
void print_center_string(char *s, int len);
void print_board(int direction);
int create_game_piece();
int keypress();
void print_direction(int direction);
void print_color_for_piece(int piece);
int move_board(int direction);
int check_win();
void move_pieces(int direction);
void move_piece(int x, int y, int direction);
void merge_pieces(int direction);
void merge_piece(int x, int y, int direction);
int out_of_bounds(int x, int y);
int has_moves_left();

// fix the tty on exit
void on_exit2()
{ /* Fixed to on_exit2 (on_exit is in non-posix stdlib.h) */
    // No system function currently
    // system("stty cooked");
}

// print the usage to the given stream
void print_usage(FILE *stream)
{
    fprintf(stream, "Usage: 2048 [-a] [-b] [-h] [-g <goal>]\n");
    fprintf(stream, "\n");
    fprintf(stream, "Options\n");
    fprintf(stream, "  -a          enable animations\n");
    fprintf(stream, "  -b          disable color output; b for boring!\n");
    fprintf(stream, "  -g <goal>   the goal piece, defaults to %d\n", goal);
    fprintf(stream, "  -h          print this message and exit\n");
}

// the game loop
int main(int argc, char **argv)
{
    int direction = -1;

    // options
    int c;
    while ((c = getopt(argc, argv, "abg:h")) != -1) {
        switch (c) {
        case 'a':
            animations = 1;
            break;
        case 'b':
            use_colors = 0;
            break;
        case 'g':
            goal = atoi(optarg);
            break;
        case 'h':
            print_usage(stdout);
            exit(0);
        case '?':
            print_usage(stderr);
            exit(1);
        }
    }

    // setup exit handler
    atexit(on_exit2);
    // No system function currently
    // system("stty raw");

    // initialize random
    srand(time(NULL));

    // setup the board
    initialize();

    // the board starts with 2 pieces
    create_game_piece();
    create_game_piece();

    // game loop
    while (1) {
        // print the board to the user
        print_board(direction);

        // block until keypress
        direction = keypress();

        // move the board
        if (!move_board(direction))
            continue;

        // inecrement moves count
        moves++;

        // check win
        if (check_win()) {
            print_board(-1);
            printf("congratulations! you've won in %d moves\r\n", moves);
            return 0;
        }

        // create a new piece
        create_game_piece();

        // check for moves left
        if (!has_moves_left()) {
            print_board(-1);
            printf("you lose! try again\r\n");
            sleep(1);
            return 1;
        }
    }
    return 0;
}

// initialize
void initialize()
{
    // clear the gameboard
    int i;
    for (i = 0; i < BOARD_SIZE * BOARD_SIZE; i++) board[i] = 0;
    moves = 0;
}

// take a 2d index and return the 1d index
int index2d(int x, int y)
{
    return BOARD_SIZE * y + x;
}

// take a 1d index and return the 2d index
int index1dX(int index)
{
    return index % BOARD_SIZE;
}
int index1dY(int index)
{
    return index / BOARD_SIZE;
}

// center a string in a given padding
void print_center_string(char *s, int len)
{
    char buf[len + 1];

    int slen = strlen(s);
    int start = (len - slen) / 2;

    int i;
    for (i = 0; i < start; i++) buf[i] = ' ';
    for (i = 0; i < slen; i++) buf[i + start] = s[i];
    for (i = start + slen; i < len; i++) buf[i] = ' ';

    buf[len] = '\0';

    printf("%s", buf);
}

// print the gameboard
void print_board(int direction)
{
    screen_clear();
    printf("2048.c - %d moves - ctrl-c to exit\r\n\r\n", moves);

    int x, y, piece;
    for (x = 0; x < BOARD_SIZE; x++) {
        printf("%c", '|');
        for (y = 0; y < BOARD_SIZE; y++) printf("%s", "----------|");
        printf("%s", "\r\n");
        printf("%c", '|');
        for (y = 0; y < BOARD_SIZE; y++) {
            piece = board[index2d(x, y)];
            print_color_for_piece(piece);
            printf("          %s|", ANSI_COLOR_RESET);
        }
        printf("%s", "\r\n");
        printf("%c", '|');
        for (y = 0; y < BOARD_SIZE; y++) {
            piece = board[index2d(x, y)];
            if (piece) {
                // convert the piece integer to a string so
                // it can be printed
                char s[10];
                snprintf(s, 10, "%d", piece);
                print_color_for_piece(piece);
                print_center_string(s, 10);
                printf("%s", ANSI_COLOR_RESET);
                printf("%c", '|');
            } else {
                printf("%s", "          |");
            }
        }
        printf("%s", "\r\n");
        printf("%c", '|');
        for (y = 0; y < BOARD_SIZE; y++) {
            piece = board[index2d(x, y)];
            print_color_for_piece(piece);
            printf("          %s|", ANSI_COLOR_RESET);
        }
        printf("%s", "\r\n");
    }
    printf("%c", '|');
    for (y = 0; y < BOARD_SIZE; y++) printf("%s", "----------|");
    printf("%s", "\r\n");

    // print the direction if given
    if (direction > -1)
        print_direction(direction);

    printf("%s", "\r\n");
    printf("%s", "\r\n");
}

// create a new piece
int create_game_piece()
{
    int i;
    // count the number of empty spots
    int numempty = 0;
    for (i = 0; i < BOARD_SIZE * BOARD_SIZE; i++)
        if (!board[i])
            numempty++;

    // fail if we don't have any spots left
    if (numempty == 0)
        return 0;

    // pick the random spot
    int r = rand() % numempty;

    // make the new piece
    int j = 0;
    for (i = 0; i < BOARD_SIZE * BOARD_SIZE; i++) {
        // skip non-empty spots
        if (board[i])
            continue;

        if (j == r) {
            board[i] = 2;
            return 1;
        }
        j++;
    }
    return 0;
}

// wait for user input and return the direction pressed
int keypress()
{
    // loop until we get valid input
    while (1) {
        if (getchar() == 3) // ctr-c
            exit(0);
        getchar();
        switch (getchar()) {
        case 'A':
            return DIRECTION_UP;
        case 'B':
            return DIRECTION_DOWN;
        case 'C':
            return DIRECTION_RIGHT;
        case 'D':
            return DIRECTION_LEFT;
        }
    }
    /* NOTREACHED */
    return -1;
}

// print a direction to stdout
void print_direction(int direction)
{
    switch (direction) {
    case DIRECTION_UP:
        printf("%s", "up");
        break;
    case DIRECTION_DOWN:
        printf("%s", "down");
        break;
    case DIRECTION_RIGHT:
        printf("%s", "right");
        break;
    case DIRECTION_LEFT:
        printf("%s", "left");
        break;
    }
}

// given a number, print the color for that piece
void print_color_for_piece(int piece)
{
    if (!use_colors)
        return;
    switch (piece) {
    case 2:
        printf("%s", ANSI_COLOR_BG_RED);
        break;
    case 4:
        printf("%s", ANSI_COLOR_BG_GREEN);
        break;
    case 8:
        printf("%s", ANSI_COLOR_BG_YELLOW);
        break;
    case 16:
        printf("%s", ANSI_COLOR_BG_BLUE);
        break;
    case 32:
        printf("%s", ANSI_COLOR_BG_MAGENTA);
        break;
    case 64:
        printf("%s", ANSI_COLOR_BG_CYAN);
        break;
    case 128:
        printf("%s", ANSI_COLOR_BG_WHITE);
        break;
    case 256:
        printf("%s", ANSI_COLOR_BG_BLUE);
        break;
    case 512:
        printf("%s", ANSI_COLOR_BG_MAGENTA);
        break;
    case 1024:
        printf("%s", ANSI_COLOR_BG_GREEN);
        break;
    case 2048:
        printf("%s", ANSI_COLOR_REVERSE);
        break;
    }
}

// move the board in a direction
// return 1 if the board moves, 0 if nothing happens
int move_board(int direction)
{
    // copy the board, so we can determine at the end if anything changed
    int i;
    for (i = 0; i < BOARD_SIZE * BOARD_SIZE; i++) _board[i] = board[i];

    // a single play in 2048 is move, merge, then move again.
    move_pieces(direction);
    merge_pieces(direction);
    move_pieces(direction);

    // check to see if the board has changed
    for (i = 0; i < BOARD_SIZE * BOARD_SIZE; i++)
        // if these are different, the board has changed
        if ((board[i] != _board[i]))
            return 1;

    return 0;
}

// check to see if we have won
int check_win()
{
    int i;
    for (i = 0; i < BOARD_SIZE * BOARD_SIZE; i++)
        if (board[i] == goal)
            return 1;
    return 0;
}

// move the pieces of a board in a given direction
void move_pieces(int direction)
{
    int x, y;
    switch (direction) {
    case DIRECTION_UP:
        for (x = 0; x < BOARD_SIZE; x++)
            for (y = 0; y < BOARD_SIZE; y++) move_piece(x, y, direction);
        break;
    case DIRECTION_DOWN:
        for (x = BOARD_SIZE - 1; x >= 0; x--)
            for (y = 0; y < BOARD_SIZE; y++) move_piece(x, y, direction);
        break;
    case DIRECTION_LEFT:
        for (x = 0; x < BOARD_SIZE; x++)
            for (y = 0; y < BOARD_SIZE; y++) move_piece(x, y, direction);
        break;
    case DIRECTION_RIGHT:
        for (x = 0; x < BOARD_SIZE; x++)
            for (y = BOARD_SIZE - 1; y >= 0; y--) move_piece(x, y, direction);
        break;
    }
}

// check if an x, y is out of bounds
int out_of_bounds(int x, int y)
{
    return x < 0 || y < 0 || x >= BOARD_SIZE || y >= BOARD_SIZE;
}

// move a piece in a direction
void move_piece(int x, int y, int direction)
{
    while (1) {
        if (out_of_bounds(x, y))
            return;
        int pieceindex = index2d(x, y);
        int piece = board[pieceindex];

        if (!piece)
            return;

        int ox, oy;
        switch (direction) {
        case DIRECTION_UP:
            ox = x - 1;
            oy = y;
            break;
        case DIRECTION_DOWN:
            ox = x + 1;
            oy = y;
            break;
        case DIRECTION_LEFT:
            ox = x;
            oy = y - 1;
            break;
        case DIRECTION_RIGHT:
            ox = x;
            oy = y + 1;
            break;
        }

        if (out_of_bounds(ox, oy))
            return;
        int opieceindex = index2d(ox, oy);
        int opiece = board[opieceindex];

        // stop trying if the other piece is something
        if (opiece)
            return;

        // swap the pieces
        board[opieceindex] = board[pieceindex];
        board[pieceindex] = 0;

        if (animations) {
            print_board(-1);
            usleep(1000 * 10);
        }

        x = ox;
        y = oy;
    }
}

// merge near pieces
void merge_pieces(int direction)
{
    int x, y;
    switch (direction) {
    case DIRECTION_UP:
        for (x = 0; x < BOARD_SIZE; x++)
            for (y = 0; y < BOARD_SIZE; y++) merge_piece(x, y, direction);
        break;
    case DIRECTION_DOWN:
        for (x = BOARD_SIZE - 1; x >= 0; x--)
            for (y = 0; y < BOARD_SIZE; y++) merge_piece(x, y, direction);
        break;
    case DIRECTION_LEFT:
        for (x = 0; x < BOARD_SIZE; x++)
            for (y = 0; y < BOARD_SIZE; y++) merge_piece(x, y, direction);
        break;
    case DIRECTION_RIGHT:
        for (x = 0; x < BOARD_SIZE; x++)
            for (y = BOARD_SIZE - 1; y >= 0; y--) merge_piece(x, y, direction);
        break;
    }
}

// merge a piece in a direction
void merge_piece(int x, int y, int direction)
{
    if (out_of_bounds(x, y))
        return;
    int pieceindex = index2d(x, y);
    int piece = board[pieceindex];

    if (!piece)
        return;

    int ox, oy;
    switch (direction) {
    case DIRECTION_UP:
        ox = x - 1;
        oy = y;
        break;
    case DIRECTION_DOWN:
        ox = x + 1;
        oy = y;
        break;
    case DIRECTION_LEFT:
        ox = x;
        oy = y - 1;
        break;
    case DIRECTION_RIGHT:
        ox = x;
        oy = y + 1;
        break;
    }

    if (out_of_bounds(ox, oy))
        return;
    int opieceindex = index2d(ox, oy);
    int opiece = board[opieceindex];

    if (!opiece)
        return;

    if (piece == opiece) {
        // merge them
        board[opieceindex] = piece * 2;
        board[pieceindex] = 0;
    }
}

// check if we have move lefts
int has_moves_left()
{
    int i;
    for (i = 0; i < BOARD_SIZE * BOARD_SIZE; i++)
        // if we have 1 empty space, we have move lefts
        if (!board[i])
            return 1;

    int x, y, ox, oy, piece, pieceindex, opiece, opieceindex;
    for (x = 0; x < BOARD_SIZE; x++) {
        for (y = 0; y < BOARD_SIZE; y++) {
            pieceindex = index2d(x, y);
            piece = board[pieceindex];

            // check north
            ox = x;
            oy = y - 1;
            if (!out_of_bounds(ox, oy)) {
                opieceindex = index2d(ox, oy);
                opiece = board[opieceindex];
                if (opiece == piece)
                    return 1;
            }
            // check south
            ox = x;
            oy = y + 1;
            if (!out_of_bounds(ox, oy)) {
                opieceindex = index2d(ox, oy);
                opiece = board[opieceindex];
                if (opiece == piece)
                    return 1;
            }
            // check east
            ox = x + 1;
            oy = y;
            if (!out_of_bounds(ox, oy)) {
                opieceindex = index2d(ox, oy);
                opiece = board[opieceindex];
                if (opiece == piece)
                    return 1;
            }
            // check west
            ox = x - 1;
            oy = y;
            if (!out_of_bounds(ox, oy)) {
                opieceindex = index2d(ox, oy);
                opiece = board[opieceindex];
                if (opiece == piece)
                    return 1;
            }
        }
    }
    return 0;
}