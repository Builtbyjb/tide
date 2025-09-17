const std = @import("std");

pub fn parse(allocator: std.mem.Allocator, file: []const u8) !void {
    // const Cmd = struct { variable: []const u8, commands: std.ArrayList([]const u8) };

    var variable_map = std.StringHashMap(std.array_list.Managed([]const u8)).init(allocator);
    defer variable_map.deinit();

    var lines = std.mem.splitScalar(u8, file, '\n');
    while (lines.next()) |line| {
        const trimmed_line = std.mem.trim(u8, line, " \t\r\n");
        if (trimmed_line.len > 1) {
            if (trimmed_line[0] == '[' and trimmed_line[trimmed_line.len - 1] == ']') {
                const table_name = trimmed_line[1 .. trimmed_line.len - 1];
                var split = std.mem.splitScalar(u8, table_name, '.');
                const first = split.next() orelse "";
                const second = split.next() orelse "";
                // How to know this is or is not a previous table end
                std.debug.print("table name: {s} - {s}\n", .{ first, second });
            } else {
                var split = std.mem.splitScalar(u8, trimmed_line, '=');
                const key = split.next() orelse "";
                const value = split.next() orelse "";
                std.debug.print("key: {s} - value: {s}\n", .{ key, value });
                // I need a function to parse the value

                var values = std.array_list.Managed([]const u8).init(allocator);
                defer values.deinit();
                values.append(value) catch unreachable;

                if (key.len > 0 and value.len > 0) {
                    try variable_map.put(key, values);
                    // try variable_map.getPtr(key).?.commands.append(allocator, value);
                }
            }
        }
    }

    var iter = variable_map.iterator();
    while (iter.next()) |entry| {
        std.debug.print("{s}", .{entry.key_ptr.*});
        for (entry.value_ptr.*.items) |item| {
            std.debug.print("{s}", .{item});
        }
        std.debug.print("\n", .{});
    }
}
