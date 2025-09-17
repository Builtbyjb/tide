const std = @import("std");
const builtin = @import("builtin");
const toml = @import("toml");

var gpa = std.heap.GeneralPurposeAllocator(.{}){};
const allocator = gpa.allocator();

const VERSION: []const u8 = "0.1.2";

fn start(arg: []const u8, watch: bool) !void {
    printTide();

    // Read config file
    const file_contents: []u8 = try std.fs.cwd().readFileAlloc(allocator, "tide.toml", 1024 * 1024);
    defer allocator.free(file_contents);

    // Parse config file
    try toml.parse(allocator, file_contents);

    std.debug.print("arg {s}\n", .{arg});
    std.debug.print("watch {}\n", .{watch});
    // Parse .toml file
    // Check os
    // Search check if the variable exits
    // Run the command(s) assigned to the variable
}

// Creates a tide.toml config file
fn init() !void {
    var cwd = std.fs.cwd();
    const tide =
        \\ root_dir = "."
        \\
        \\ [os.unix]
        \\ dev = []
        \\
        \\ [os.windows]
        \\ dev = []
        \\
        \\
        \\ [exclude]
        \\ directory = [".git"]
        \\ file = ["README.md"]
        \\ extension = ["toml"]
    ;

    const file = cwd.openFile("tide.toml", .{}) catch |err| {
        if (err == error.FileNotFound) {
            // Create .toml file
            var file = try cwd.createFile("tide.toml", .{});
            defer file.close();
            return try file.writeAll(tide);
        } else {
            std.debug.print("Error opening file: {}\n", .{err});
            return;
        }
    };
    std.debug.print("Created tide.toml\n", .{});
    defer file.close();
}

fn printUsage() void {
    std.debug.print(
        \\Usage: tide [options] [command]
        \\
        \\Options:
        \\  init               Create a tide configuration file
        \\  run [command]      Run commands
        \\      -w, --watch    Watch for changes and re-run commands
        \\  -h, --help         Display this help message
        \\  -v, --version      Display the version number
        \\
    , .{});
}

fn printTide() void {
    std.debug.print(
        \\     __   _      __
        \\    / /_ (_)____/ /___
        \\   / __// // __  // _ \
        \\  / /_ / // /_/ //  __/
        \\  \__//_/ \__,_/ \___/  v{s}
        \\
        \\  Press Ctrl + C to exit
    , .{VERSION});
    std.debug.print("\n", .{});
}

pub fn main() !void {
    var args_iterator = try std.process.ArgIterator.initWithAllocator(allocator);
    defer _ = args_iterator.deinit();

    var args_list = std.array_list.Managed([]const u8).init(allocator);
    defer args_list.deinit();

    while (args_iterator.next()) |arg| {
        args_list.append(arg) catch unreachable;
    }

    switch (args_list.items.len) {
        2 => {
            const arg2 = args_list.items[1];
            if (std.mem.eql(u8, arg2, "init")) {
                try init();
            } else if (std.mem.eql(u8, arg2, "--version") or std.mem.eql(u8, arg2, "-v")) {
                std.debug.print("tide v{s}\n", .{VERSION});
            } else if (std.mem.eql(u8, arg2, "--help") or std.mem.eql(u8, arg2, "-h")) {
                printUsage();
            } else {
                printUsage();
            }
        },
        3 => {
            if (std.mem.eql(u8, args_list.items[1], "run")) {
                try start(args_list.items[2], false);
            } else {
                printUsage();
            }
        },
        4 => {
            if (std.mem.eql(u8, args_list.items[1], "run")) {
                const arg3 = args_list.items[3];
                if (std.mem.eql(u8, arg3, "--watch") or std.mem.eql(u8, arg3, "-w")) {
                    try start(args_list.items[2], true);
                } else {
                    printUsage();
                }
            } else {
                printUsage();
            }
        },
        else => printUsage(),
    }
}

// test "simple test" {
//     const gpa = std.testing.allocator;
//     var list: std.ArrayList(i32) = .empty;
//     defer list.deinit(gpa); // Try commenting this out and see if zig detects the memory leak!
//     try list.append(gpa, 42);
//     try std.testing.expectEqual(@as(i32, 42), list.pop());
// }

// test "fuzz example" {
//     const Context = struct {
//         fn testOne(context: @This(), input: []const u8) anyerror!void {
//             _ = context;
//             // Try passing `--fuzz` to `zig build test` and see if it manages to fail this test case!
//             try std.testing.expect(!std.mem.eql(u8, "canyoufindme", input));
//         }
//     };
//     try std.testing.fuzz(Context{}, Context.testOne, .{});
// }
