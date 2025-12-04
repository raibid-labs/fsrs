// Fusabi Standard Library
// Provides built-in functions for List, String, Map, Array, and Option operations

pub mod array;
pub mod async_ops;
pub mod commands;
pub mod config;
pub mod console;
pub mod events;
pub mod list;
pub mod map;
pub mod option;
pub mod math;
pub mod result;
pub mod print;
pub mod script;
pub mod string;
pub mod process;
pub mod terminal_control;
pub mod terminal_info;
pub mod ui_formatting;
pub mod url;
pub mod time;

#[cfg(feature = "json")]
pub mod json;

#[cfg(feature = "osc")]
pub mod net;

use crate::value::Value;
use crate::vm::{Vm, VmError};
use std::sync::Mutex;
use std::collections::HashMap;
use std::sync::Arc;

/// Register all standard library functions into the VM
pub fn register_stdlib(vm: &mut Vm) {
    // 1. Register functions in HostRegistry
    {
        let mut registry = vm.host_registry.lock().unwrap();

        // List functions
        registry.register("List.length", |_vm, args| {
            wrap_unary(args, list::list_length)
        });
        registry.register("List.head", |_vm, args| wrap_unary(args, list::list_head));
        registry.register("List.tail", |_vm, args| wrap_unary(args, list::list_tail));
        registry.register("List.reverse", |_vm, args| {
            wrap_unary(args, list::list_reverse)
        });
        registry.register("List.isEmpty", |_vm, args| {
            wrap_unary(args, list::list_is_empty)
        });
        registry.register("List.append", |_vm, args| {
            wrap_binary(args, list::list_append)
        });
        registry.register("List.concat", |_vm, args| {
            wrap_unary(args, list::list_concat)
        });
        registry.register("List.map", list::list_map);
        registry.register("List.iter", list::list_iter);
        registry.register("List.filter", list::list_filter);
        registry.register("List.fold", list::list_fold);
        registry.register("List.exists", list::list_exists);
        registry.register("List.find", list::list_find);
        registry.register("List.tryFind", list::list_try_find);
        registry.register("List.nth", |_vm, args| {
            wrap_binary(args, list::list_nth)
        });
        registry.register("List.mapi", list::list_mapi);

        // Async functions
        registry.register("Async.Return", async_ops::async_return);
        registry.register("Async.ReturnFrom", async_ops::async_return_from);
        registry.register("Async.Bind", async_ops::async_bind);
        registry.register("Async.Delay", async_ops::async_delay);
        registry.register("Async.Zero", async_ops::async_zero);
        registry.register("Async.Combine", async_ops::async_combine);
        registry.register("Async.RunSynchronously", async_ops::async_run_synchronously);
        // Internal helper
        registry.register("Async.Internal.CombineHelper", async_ops::async_combine_helper);

        // Array functions
        registry.register("Array.length", |_vm, args| {
            wrap_unary(args, array::array_length)
        });
        registry.register("Array.isEmpty", |_vm, args| {
            wrap_unary(args, array::array_is_empty)
        });
        registry.register("Array.get", |_vm, args| {
            wrap_binary(args, array::array_get)
        });
        registry.register("Array.set", |_vm, args| {
            wrap_ternary(args, array::array_set)
        });
        registry.register("Array.ofList", |_vm, args| {
            wrap_unary(args, array::array_of_list)
        });
        registry.register("Array.toList", |_vm, args| {
            wrap_unary(args, array::array_to_list)
        });
        registry.register("Array.init", array::array_init);
        registry.register("Array.create", |_vm, args| {
            wrap_binary(args, array::array_create)
        });

        // String functions
        registry.register("String.length", |_vm, args| {
            wrap_unary(args, string::string_length)
        });
        registry.register("String.trim", |_vm, args| {
            wrap_unary(args, string::string_trim)
        });
        registry.register("String.toLower", |_vm, args| {
            wrap_unary(args, string::string_to_lower)
        });
        registry.register("String.toUpper", |_vm, args| {
            wrap_unary(args, string::string_to_upper)
        });
        registry.register("String.split", |_vm, args| {
            wrap_binary(args, string::string_split)
        });
        registry.register("String.concat", |_vm, args| {
            wrap_unary(args, string::string_concat)
        });
        registry.register("String.contains", |_vm, args| {
            wrap_binary(args, string::string_contains)
        });
        registry.register("String.startsWith", |_vm, args| {
            wrap_binary(args, string::string_starts_with)
        });
        registry.register("String.endsWith", |_vm, args| {
            wrap_binary(args, string::string_ends_with)
        });
        registry.register("String.format", |_vm, args| {
            wrap_binary(args, string::string_format)
        });
        registry.register("sprintf", |_vm, args| {
            wrap_binary(args, string::string_format)
        });

        // Print functions (global functions, not in a module)
        registry.register("print", |_vm, args| {
            wrap_unary(args, print::print_value)
        });
        registry.register("printfn", |_vm, args| {
            wrap_unary(args, print::printfn_value)
        });

        // Math functions
        registry.register("Math.pi", |_vm, args| {
            wrap_unary(args, math::math_pi)
        });
        registry.register("Math.e", |_vm, args| {
            wrap_unary(args, math::math_e)
        });
        registry.register("Math.abs", |_vm, args| {
            wrap_unary(args, math::math_abs)
        });
        registry.register("Math.sqrt", |_vm, args| {
            wrap_unary(args, math::math_sqrt)
        });
        registry.register("Math.pow", |_vm, args| {
            wrap_binary(args, math::math_pow)
        });
        registry.register("Math.max", |_vm, args| {
            wrap_binary(args, math::math_max)
        });
        registry.register("Math.min", |_vm, args| {
            wrap_binary(args, math::math_min)
        });
        registry.register("Math.sin", |_vm, args| {
            wrap_unary(args, math::math_sin)
        });
        registry.register("Math.cos", |_vm, args| {
            wrap_unary(args, math::math_cos)
        });
        registry.register("Math.tan", |_vm, args| {
            wrap_unary(args, math::math_tan)
        });
        registry.register("Math.asin", |_vm, args| {
            wrap_unary(args, math::math_asin)
        });
        registry.register("Math.acos", |_vm, args| {
            wrap_unary(args, math::math_acos)
        });
        registry.register("Math.atan", |_vm, args| {
            wrap_unary(args, math::math_atan)
        });
        registry.register("Math.atan2", |_vm, args| {
            wrap_binary(args, math::math_atan2)
        });
        registry.register("Math.log", |_vm, args| {
            wrap_unary(args, math::math_log)
        });
        registry.register("Math.log10", |_vm, args| {
            wrap_unary(args, math::math_log10)
        });
        registry.register("Math.exp", |_vm, args| {
            wrap_unary(args, math::math_exp)
        });
        registry.register("Math.floor", |_vm, args| {
            wrap_unary(args, math::math_floor)
        });
        registry.register("Math.ceil", |_vm, args| {
            wrap_unary(args, math::math_ceil)
        });
        registry.register("Math.round", |_vm, args| {
            wrap_unary(args, math::math_round)
        });
        registry.register("Math.truncate", |_vm, args| {
            wrap_unary(args, math::math_truncate)
        });

        // Map functions
        registry.register("Map.empty", |_vm, args| {
            wrap_unary(args, map::map_empty)
        });
        registry.register("Map.add", |_vm, args| {
            wrap_ternary(args, map::map_add)
        });
        registry.register("Map.remove", |_vm, args| {
            wrap_binary(args, map::map_remove)
        });
        registry.register("Map.find", |_vm, args| {
            wrap_binary(args, map::map_find)
        });
        registry.register("Map.tryFind", |_vm, args| {
            wrap_binary(args, map::map_try_find)
        });
        registry.register("Map.containsKey", |_vm, args| {
            wrap_binary(args, map::map_contains_key)
        });
        registry.register("Map.isEmpty", |_vm, args| {
            wrap_unary(args, map::map_is_empty)
        });
        registry.register("Map.count", |_vm, args| {
            wrap_unary(args, map::map_count)
        });
        registry.register("Map.ofList", |_vm, args| {
            wrap_unary(args, map::map_of_list)
        });
        registry.register("Map.toList", |_vm, args| {
            wrap_unary(args, map::map_to_list)
        });
        registry.register("Map.map", map::map_map);
        registry.register("Map.iter", map::map_iter);

        // Option functions
        registry.register("Option.isSome", |_vm, args| {
            wrap_unary(args, option::option_is_some)
        });
        registry.register("Option.isNone", |_vm, args| {
            wrap_unary(args, option::option_is_none)
        });
        registry.register("Option.defaultValue", |_vm, args| {
            wrap_binary(args, option::option_default_value)
        });
        registry.register("Option.defaultWith", option::option_default_with);
        registry.register("Option.map", option::option_map);
        registry.register("Option.bind", option::option_bind);
        registry.register("Option.iter", option::option_iter);
        registry.register("Option.map2", option::option_map2);
        registry.register("Option.orElse", |_vm, args| {
            wrap_binary(args, option::option_or_else)
        });

        // Option constructors - Some and None
        registry.register("Some", |_vm, args| {
            if args.len() != 1 {
                return Err(VmError::Runtime(format!(
                    "Some expects 1 argument, got {}",
                    args.len()
                )));
            }
            Ok(Value::Variant {
                type_name: "Option".to_string(),
                variant_name: "Some".to_string(),
                fields: vec![args[0].clone()],
            })
        });
        registry.register("None", |_vm, args| {
            if !args.is_empty() {
                return Err(VmError::Runtime(format!(
                    "None expects 0 arguments, got {}",
                    args.len()
                )));
            }
            Ok(Value::Variant {
                type_name: "Option".to_string(),
                variant_name: "None".to_string(),
                fields: vec![],
            })
        });

        // Result functions
        registry.register("Result.isOk", |_vm, args| {
            wrap_unary(args, result::result_is_ok)
        });
        registry.register("Result.isError", |_vm, args| {
            wrap_unary(args, result::result_is_error)
        });
        registry.register("Result.defaultValue", |_vm, args| {
            wrap_binary(args, result::result_default_value)
        });
        registry.register("Result.defaultWith", result::result_default_with);
        registry.register("Result.map", result::result_map);
        registry.register("Result.mapError", result::result_map_error);
        registry.register("Result.bind", result::result_bind);
        registry.register("Result.iter", result::result_iter);

        // Result constructors - Ok and Error
        registry.register("Ok", |_vm, args| {
            if args.len() != 1 {
                return Err(VmError::Runtime(format!(
                    "Ok expects 1 argument, got {}",
                    args.len()
                )));
            }
            Ok(Value::Variant {
                type_name: "Result".to_string(),
                variant_name: "Ok".to_string(),
                fields: vec![args[0].clone()],
            })
        });
        registry.register("Error", |_vm, args| {
            if args.len() != 1 {
                return Err(VmError::Runtime(format!(
                    "Error expects 1 argument, got {}",
                    args.len()
                )));
            }
            Ok(Value::Variant {
                type_name: "Result".to_string(),
                variant_name: "Error".to_string(),
                fields: vec![args[0].clone()],
            })
        });

        // Process functions
        registry.register("Process.run", |_vm, args| {
            wrap_binary(args, process::process_run)
        });
        registry.register("Process.runShell", |_vm, args| {
            wrap_unary(args, process::process_run_shell)
        });
        registry.register("Process.env", |_vm, args| {
            wrap_unary(args, process::process_env)
        });
        registry.register("Process.setEnv", |_vm, args| {
            wrap_binary(args, process::process_set_env)
        });
        registry.register("Process.cwd", |_vm, args| {
            wrap_unary(args, process::process_cwd)
        });

        // Config functions
        registry.register("Config.define", config::config_define);
        registry.register("Config.get", |_vm, args| {
            wrap_unary(args, config::config_get)
        });
        registry.register("Config.getOr", |_vm, args| {
            wrap_binary(args, config::config_get_or)
        });
        registry.register("Config.set", config::config_set);
        registry.register("Config.has", |_vm, args| {
            wrap_unary(args, config::config_has)
        });
        registry.register("Config.list", |_vm, args| {
            wrap_unary(args, config::config_list)
        });
        registry.register("Config.reset", |_vm, args| {
            wrap_unary(args, config::config_reset)
        });

        // Time functions
        registry.register("Time.now", |_vm, args| {
            wrap_unary(args, time::time_now)
        });
        registry.register("Time.nowSeconds", |_vm, args| {
            wrap_unary(args, time::time_now_seconds)
        });
        registry.register("Time.format", |_vm, args| {
            wrap_binary(args, time::time_format)
        });
        registry.register("Time.parse", |_vm, args| {
            wrap_binary(args, time::time_parse)
        });

        // Url functions
        registry.register("Url.parse", |_vm, args| {
            wrap_unary(args, url::url_parse)
        });
        registry.register("Url.isValid", |_vm, args| {
            wrap_unary(args, url::url_is_valid)
        });
        registry.register("Url.encode", |_vm, args| {
            wrap_unary(args, url::url_encode)
        });
        registry.register("Url.decode", |_vm, args| {
            wrap_unary(args, url::url_decode)
        });

        // Json functions (if json feature is enabled)
        #[cfg(feature = "json")]
        {
            registry.register("Json.parse", |_vm, args| {
                wrap_unary(args, json::json_parse)
            });
            registry.register("Json.stringify", |_vm, args| {
                wrap_unary(args, json::json_stringify)
            });
            registry.register("Json.stringifyPretty", |_vm, args| {
                wrap_unary(args, json::json_stringify_pretty)
            });
        }

        // Net.Osc functions (if osc feature is enabled)
        #[cfg(feature = "osc")]
        {
            registry.register("Osc.client", net::osc::osc_client);
            registry.register("Osc.send", net::osc::osc_send);
        }

        // Events functions
        registry.register("Events.on", events::events_on);
        registry.register("Events.off", |_vm, args| {
            wrap_unary(args, events::events_off)
        });
        registry.register("Events.emit", events::events_emit);
        registry.register("Events.emitAsync", events::events_emit_async);
        registry.register("Events.once", events::events_once);
        registry.register("Events.clear", |_vm, args| {
            wrap_unary(args, events::events_clear)
        });
        registry.register("Events.clearAll", |_vm, args| {
            wrap_unary(args, events::events_clear_all)
        });
        registry.register("Events.handlers", |_vm, args| {
            wrap_unary(args, events::events_handlers)
        });
        registry.register("Events.list", |_vm, args| {
            wrap_unary(args, events::events_list)
        });

        // TerminalInfo functions
        registry.register("TerminalInfo.getForegroundProcess", |_vm, args| {
            wrap_unary(args, terminal_info::get_foreground_process)
        });
        registry.register("TerminalInfo.getCurrentWorkingDir", |_vm, args| {
            wrap_unary(args, terminal_info::get_current_working_dir)
        });
        registry.register("TerminalInfo.getLine", |_vm, args| {
            wrap_unary(args, terminal_info::get_line)
        });
        registry.register("TerminalInfo.getLines", |_vm, args| {
            wrap_binary(args, terminal_info::get_lines)
        });
        registry.register("TerminalInfo.getWindowTitle", |_vm, args| {
            wrap_unary(args, terminal_info::get_window_title)
        });
        registry.register("TerminalInfo.getTabTitle", |_vm, args| {
            wrap_unary(args, terminal_info::get_tab_title)
        });
        registry.register("TerminalInfo.getTerminalSize", |_vm, args| {
            wrap_unary(args, terminal_info::get_terminal_size)
        });

        // TerminalControl functions
        registry.register("TerminalControl.sendText", |_vm, args| {
            wrap_unary(args, terminal_control::send_text)
        });
        registry.register("TerminalControl.sendKeys", |_vm, args| {
            wrap_unary(args, terminal_control::send_keys)
        });
        registry.register("TerminalControl.splitHorizontal", |_vm, args| {
            wrap_unary(args, terminal_control::split_horizontal)
        });
        registry.register("TerminalControl.splitVertical", |_vm, args| {
            wrap_unary(args, terminal_control::split_vertical)
        });
        registry.register("TerminalControl.closePane", |_vm, args| {
            wrap_unary(args, terminal_control::close_pane)
        });
        registry.register("TerminalControl.focusPane", |_vm, args| {
            wrap_unary(args, terminal_control::focus_pane)
        });
        registry.register("TerminalControl.createTab", |_vm, args| {
            wrap_unary(args, terminal_control::create_tab)
        });
        registry.register("TerminalControl.closeTab", |_vm, args| {
            wrap_unary(args, terminal_control::close_tab)
        });
        registry.register("TerminalControl.setTabTitle", |_vm, args| {
            wrap_binary(args, terminal_control::set_tab_title)
        });
        registry.register("TerminalControl.showToast", |_vm, args| {
            wrap_unary(args, terminal_control::show_toast)
        });

        // UIFormatting functions
        registry.register("UIFormatting.onFormatTab", ui_formatting::on_format_tab);
        registry.register("UIFormatting.onFormatStatusLeft", ui_formatting::on_format_status_left);
        registry.register("UIFormatting.onFormatStatusRight", ui_formatting::on_format_status_right);
        registry.register("UIFormatting.removeFormatter", |_vm, args| {
            ui_formatting::remove_formatter(args)
        });
        registry.register("UIFormatting.clearFormatters", |_vm, args| {
            ui_formatting::clear_formatters(args)
        });

        // Commands functions
        registry.register("Commands.register", commands::commands_register);
        registry.register("Commands.registerMany", commands::commands_register_many);
        registry.register("Commands.unregister", commands::commands_unregister);
        registry.register("Commands.list", commands::commands_list);
        registry.register("Commands.getById", commands::commands_get_by_id);
        registry.register("Commands.invoke", commands::commands_invoke);

        // Script functions
        registry.register("Script.eval", script::script_eval);
        registry.register("Script.evalToString", script::script_eval_to_string);

        // Console functions
        registry.register("Console.readLine", |_vm, args| {
            wrap_unary(args, console::console_read_line)
        });
        registry.register("Console.readKey", |_vm, args| {
            wrap_unary(args, console::console_read_key)
        });
        registry.register("Console.write", |_vm, args| {
            wrap_unary(args, console::console_write)
        });
        registry.register("Console.writeLine", |_vm, args| {
            wrap_unary(args, console::console_write_line)
        });
        registry.register("Console.clear", |_vm, args| {
            wrap_unary(args, console::console_clear)
        });
    }

    // 2. Populate Globals with Module Records

    // Helper to create NativeFn value
    let native = |name: &str, arity: u8| Value::NativeFn {
        name: name.to_string(),
        arity,
        args: vec![],
    };

    // List Module
    let mut list_fields = HashMap::new();
    list_fields.insert("length".to_string(), native("List.length", 1));
    list_fields.insert("head".to_string(), native("List.head", 1));
    list_fields.insert("tail".to_string(), native("List.tail", 1));
    list_fields.insert("reverse".to_string(), native("List.reverse", 1));
    list_fields.insert("isEmpty".to_string(), native("List.isEmpty", 1));
    list_fields.insert("append".to_string(), native("List.append", 2));
    list_fields.insert("concat".to_string(), native("List.concat", 1));
    list_fields.insert("map".to_string(), native("List.map", 2));
    list_fields.insert("iter".to_string(), native("List.iter", 2));
    list_fields.insert("filter".to_string(), native("List.filter", 2));
    list_fields.insert("fold".to_string(), native("List.fold", 3));
    list_fields.insert("exists".to_string(), native("List.exists", 2));
    list_fields.insert("find".to_string(), native("List.find", 2));
    list_fields.insert("tryFind".to_string(), native("List.tryFind", 2));
    list_fields.insert("nth".to_string(), native("List.nth", 2));
    list_fields.insert("mapi".to_string(), native("List.mapi", 2));
    vm.globals.insert(
        "List".to_string(),
        Value::Record(Arc::new(Mutex::new(list_fields))),
    );

    // String Module
    let mut string_fields = HashMap::new();
    string_fields.insert("length".to_string(), native("String.length", 1));
    string_fields.insert("trim".to_string(), native("String.trim", 1));
    string_fields.insert("toLower".to_string(), native("String.toLower", 1));
    string_fields.insert("toUpper".to_string(), native("String.toUpper", 1));
    string_fields.insert("split".to_string(), native("String.split", 2));
    string_fields.insert("concat".to_string(), native("String.concat", 1));
    string_fields.insert("contains".to_string(), native("String.contains", 2));
    string_fields.insert("startsWith".to_string(), native("String.startsWith", 2));
    string_fields.insert("endsWith".to_string(), native("String.endsWith", 2));
    string_fields.insert("format".to_string(), native("String.format", 2));
    vm.globals.insert(
        "String".to_string(),
        Value::Record(Arc::new(Mutex::new(string_fields))),
    );

    // Register sprintf as a global alias for String.format
    vm.globals.insert("sprintf".to_string(), native("sprintf", 2));

    // Register print functions as globals
    vm.globals.insert("print".to_string(), native("print", 1));
    vm.globals.insert("printfn".to_string(), native("printfn", 1));

    // Math Module
    let mut math_fields = HashMap::new();
    math_fields.insert("pi".to_string(), native("Math.pi", 1));
    math_fields.insert("e".to_string(), native("Math.e", 1));
    math_fields.insert("abs".to_string(), native("Math.abs", 1));
    math_fields.insert("sqrt".to_string(), native("Math.sqrt", 1));
    math_fields.insert("pow".to_string(), native("Math.pow", 2));
    math_fields.insert("max".to_string(), native("Math.max", 2));
    math_fields.insert("min".to_string(), native("Math.min", 2));
    math_fields.insert("sin".to_string(), native("Math.sin", 1));
    math_fields.insert("cos".to_string(), native("Math.cos", 1));
    math_fields.insert("tan".to_string(), native("Math.tan", 1));
    math_fields.insert("asin".to_string(), native("Math.asin", 1));
    math_fields.insert("acos".to_string(), native("Math.acos", 1));
    math_fields.insert("atan".to_string(), native("Math.atan", 1));
    math_fields.insert("atan2".to_string(), native("Math.atan2", 2));
    math_fields.insert("log".to_string(), native("Math.log", 1));
    math_fields.insert("log10".to_string(), native("Math.log10", 1));
    math_fields.insert("exp".to_string(), native("Math.exp", 1));
    math_fields.insert("floor".to_string(), native("Math.floor", 1));
    math_fields.insert("ceil".to_string(), native("Math.ceil", 1));
    math_fields.insert("round".to_string(), native("Math.round", 1));
    math_fields.insert("truncate".to_string(), native("Math.truncate", 1));
    vm.globals.insert(
        "Math".to_string(),
        Value::Record(Arc::new(Mutex::new(math_fields))),
    );

    // Array Module
    let mut array_fields = HashMap::new();
    array_fields.insert("length".to_string(), native("Array.length", 1));
    array_fields.insert("isEmpty".to_string(), native("Array.isEmpty", 1));
    array_fields.insert("get".to_string(), native("Array.get", 2));
    array_fields.insert("set".to_string(), native("Array.set", 3));
    array_fields.insert("ofList".to_string(), native("Array.ofList", 1));
    array_fields.insert("toList".to_string(), native("Array.toList", 1));
    array_fields.insert("init".to_string(), native("Array.init", 2));
    array_fields.insert("create".to_string(), native("Array.create", 2));
    vm.globals.insert(
        "Array".to_string(),
        Value::Record(Arc::new(Mutex::new(array_fields))),
    );

    // Map Module
    let mut map_fields = HashMap::new();
    map_fields.insert("empty".to_string(), native("Map.empty", 1));
    map_fields.insert("add".to_string(), native("Map.add", 3));
    map_fields.insert("remove".to_string(), native("Map.remove", 2));
    map_fields.insert("find".to_string(), native("Map.find", 2));
    map_fields.insert("tryFind".to_string(), native("Map.tryFind", 2));
    map_fields.insert("containsKey".to_string(), native("Map.containsKey", 2));
    map_fields.insert("isEmpty".to_string(), native("Map.isEmpty", 1));
    map_fields.insert("count".to_string(), native("Map.count", 1));
    map_fields.insert("ofList".to_string(), native("Map.ofList", 1));
    map_fields.insert("toList".to_string(), native("Map.toList", 1));
    map_fields.insert("map".to_string(), native("Map.map", 2));
    map_fields.insert("iter".to_string(), native("Map.iter", 2));
    vm.globals.insert(
        "Map".to_string(),
        Value::Record(Arc::new(Mutex::new(map_fields))),
    );

    // Option Module
    let mut option_fields = HashMap::new();
    option_fields.insert("isSome".to_string(), native("Option.isSome", 1));
    option_fields.insert("isNone".to_string(), native("Option.isNone", 1));
    option_fields.insert("defaultValue".to_string(), native("Option.defaultValue", 2));
    option_fields.insert("defaultWith".to_string(), native("Option.defaultWith", 2));
    option_fields.insert("map".to_string(), native("Option.map", 2));
    option_fields.insert("bind".to_string(), native("Option.bind", 2));
    option_fields.insert("iter".to_string(), native("Option.iter", 2));
    option_fields.insert("map2".to_string(), native("Option.map2", 3));
    option_fields.insert("orElse".to_string(), native("Option.orElse", 2));
    vm.globals.insert(
        "Option".to_string(),
        Value::Record(Arc::new(Mutex::new(option_fields))),
    );

    // Register Option constructors as globals
    vm.globals.insert("Some".to_string(), native("Some", 1));
    vm.globals.insert("None".to_string(), native("None", 0));

    // Result Module
    let mut result_fields = HashMap::new();
    result_fields.insert("isOk".to_string(), native("Result.isOk", 1));
    result_fields.insert("isError".to_string(), native("Result.isError", 1));
    result_fields.insert("defaultValue".to_string(), native("Result.defaultValue", 2));
    result_fields.insert("defaultWith".to_string(), native("Result.defaultWith", 2));
    result_fields.insert("map".to_string(), native("Result.map", 2));
    result_fields.insert("mapError".to_string(), native("Result.mapError", 2));
    result_fields.insert("bind".to_string(), native("Result.bind", 2));
    result_fields.insert("iter".to_string(), native("Result.iter", 2));
    vm.globals.insert(
        "Result".to_string(),
        Value::Record(Arc::new(Mutex::new(result_fields))),
    );

    // Register Result constructors as globals
    vm.globals.insert("Ok".to_string(), native("Ok", 1));
    vm.globals.insert("Error".to_string(), native("Error", 1));

    // Process Module
    let mut process_fields = HashMap::new();
    process_fields.insert("run".to_string(), native("Process.run", 2));
    process_fields.insert("runShell".to_string(), native("Process.runShell", 1));
    process_fields.insert("env".to_string(), native("Process.env", 1));
    process_fields.insert("setEnv".to_string(), native("Process.setEnv", 2));
    process_fields.insert("cwd".to_string(), native("Process.cwd", 1));
    vm.globals.insert(
        "Process".to_string(),
        Value::Record(Arc::new(Mutex::new(process_fields))),
    );

    // Config Module
    let mut config_fields = HashMap::new();
    config_fields.insert("define".to_string(), native("Config.define", 1));
    config_fields.insert("get".to_string(), native("Config.get", 1));
    config_fields.insert("getOr".to_string(), native("Config.getOr", 2));
    config_fields.insert("set".to_string(), native("Config.set", 2));
    config_fields.insert("has".to_string(), native("Config.has", 1));
    config_fields.insert("list".to_string(), native("Config.list", 1));
    config_fields.insert("reset".to_string(), native("Config.reset", 1));
    vm.globals.insert(
        "Config".to_string(),
        Value::Record(Arc::new(Mutex::new(config_fields))),
    );

    // Time Module
    let mut time_fields = HashMap::new();
    time_fields.insert("now".to_string(), native("Time.now", 1));
    time_fields.insert("nowSeconds".to_string(), native("Time.nowSeconds", 1));
    time_fields.insert("format".to_string(), native("Time.format", 2));
    time_fields.insert("parse".to_string(), native("Time.parse", 2));
    vm.globals.insert(
        "Time".to_string(),
        Value::Record(Arc::new(Mutex::new(time_fields))),
    );

    // Url Module
    let mut url_fields = HashMap::new();
    url_fields.insert("parse".to_string(), native("Url.parse", 1));
    url_fields.insert("isValid".to_string(), native("Url.isValid", 1));
    url_fields.insert("encode".to_string(), native("Url.encode", 1));
    url_fields.insert("decode".to_string(), native("Url.decode", 1));
    vm.globals.insert(
        "Url".to_string(),
        Value::Record(Arc::new(Mutex::new(url_fields))),
    );

    // Json Module (if json feature is enabled)
    #[cfg(feature = "json")]
    {
        let mut json_fields = HashMap::new();
        json_fields.insert("parse".to_string(), native("Json.parse", 1));
        json_fields.insert("stringify".to_string(), native("Json.stringify", 1));
        json_fields.insert(
            "stringifyPretty".to_string(),
            native("Json.stringifyPretty", 1),
        );
        vm.globals.insert(
            "Json".to_string(),
            Value::Record(Arc::new(Mutex::new(json_fields))),
        );
    }

    // Osc Module (if osc feature is enabled)
    #[cfg(feature = "osc")]
    {
        let mut osc_fields = HashMap::new();
        osc_fields.insert("client".to_string(), native("Osc.client", 2));
        osc_fields.insert("send".to_string(), native("Osc.send", 3));
        vm.globals.insert(
            "Osc".to_string(),
            Value::Record(Arc::new(Mutex::new(osc_fields))),
        );
    }

    // Events Module
    let mut events_fields = HashMap::new();
    events_fields.insert("on".to_string(), native("Events.on", 2));
    events_fields.insert("off".to_string(), native("Events.off", 1));
    events_fields.insert("emit".to_string(), native("Events.emit", 2));
    events_fields.insert("emitAsync".to_string(), native("Events.emitAsync", 2));
    events_fields.insert("once".to_string(), native("Events.once", 2));
    events_fields.insert("clear".to_string(), native("Events.clear", 1));
    events_fields.insert("clearAll".to_string(), native("Events.clearAll", 1));
    events_fields.insert("handlers".to_string(), native("Events.handlers", 1));
    events_fields.insert("list".to_string(), native("Events.list", 1));
    vm.globals.insert(
        "Events".to_string(),
        Value::Record(Arc::new(Mutex::new(events_fields))),
    );

    // TerminalInfo Module
    let mut terminal_info_fields = HashMap::new();
    terminal_info_fields.insert("getForegroundProcess".to_string(), native("TerminalInfo.getForegroundProcess", 1));
    terminal_info_fields.insert("getCurrentWorkingDir".to_string(), native("TerminalInfo.getCurrentWorkingDir", 1));
    terminal_info_fields.insert("getLine".to_string(), native("TerminalInfo.getLine", 1));
    terminal_info_fields.insert("getLines".to_string(), native("TerminalInfo.getLines", 2));
    terminal_info_fields.insert("getWindowTitle".to_string(), native("TerminalInfo.getWindowTitle", 1));
    terminal_info_fields.insert("getTabTitle".to_string(), native("TerminalInfo.getTabTitle", 1));
    terminal_info_fields.insert("getTerminalSize".to_string(), native("TerminalInfo.getTerminalSize", 1));
    vm.globals.insert(
        "TerminalInfo".to_string(),
        Value::Record(Arc::new(Mutex::new(terminal_info_fields))),
    );

    // TerminalControl Module
    let mut terminal_control_fields = HashMap::new();
    terminal_control_fields.insert("sendText".to_string(), native("TerminalControl.sendText", 1));
    terminal_control_fields.insert("sendKeys".to_string(), native("TerminalControl.sendKeys", 1));
    terminal_control_fields.insert("splitHorizontal".to_string(), native("TerminalControl.splitHorizontal", 1));
    terminal_control_fields.insert("splitVertical".to_string(), native("TerminalControl.splitVertical", 1));
    terminal_control_fields.insert("closePane".to_string(), native("TerminalControl.closePane", 1));
    terminal_control_fields.insert("focusPane".to_string(), native("TerminalControl.focusPane", 1));
    terminal_control_fields.insert("createTab".to_string(), native("TerminalControl.createTab", 1));
    terminal_control_fields.insert("closeTab".to_string(), native("TerminalControl.closeTab", 1));
    terminal_control_fields.insert("setTabTitle".to_string(), native("TerminalControl.setTabTitle", 2));
    terminal_control_fields.insert("showToast".to_string(), native("TerminalControl.showToast", 1));
    vm.globals.insert(
        "TerminalControl".to_string(),
        Value::Record(Arc::new(Mutex::new(terminal_control_fields))),
    );

    // UIFormatting Module
    let mut ui_formatting_fields = HashMap::new();
    ui_formatting_fields.insert("onFormatTab".to_string(), native("UIFormatting.onFormatTab", 1));
    ui_formatting_fields.insert("onFormatStatusLeft".to_string(), native("UIFormatting.onFormatStatusLeft", 1));
    ui_formatting_fields.insert("onFormatStatusRight".to_string(), native("UIFormatting.onFormatStatusRight", 1));
    ui_formatting_fields.insert("removeFormatter".to_string(), native("UIFormatting.removeFormatter", 1));
    ui_formatting_fields.insert("clearFormatters".to_string(), native("UIFormatting.clearFormatters", 1));
    vm.globals.insert(
        "UIFormatting".to_string(),
        Value::Record(Arc::new(Mutex::new(ui_formatting_fields))),
    );

    // Commands Module
    let mut commands_fields = HashMap::new();
    commands_fields.insert("register".to_string(), native("Commands.register", 1));
    commands_fields.insert("registerMany".to_string(), native("Commands.registerMany", 1));
    commands_fields.insert("unregister".to_string(), native("Commands.unregister", 1));
    commands_fields.insert("list".to_string(), native("Commands.list", 1));
    commands_fields.insert("getById".to_string(), native("Commands.getById", 1));
    commands_fields.insert("invoke".to_string(), native("Commands.invoke", 1));
    vm.globals.insert(
        "Commands".to_string(),
        Value::Record(Arc::new(Mutex::new(commands_fields))),
    );

    // Console Module
    let mut console_fields = HashMap::new();
    console_fields.insert("readLine".to_string(), native("Console.readLine", 1));
    console_fields.insert("readKey".to_string(), native("Console.readKey", 1));
    console_fields.insert("write".to_string(), native("Console.write", 1));
    console_fields.insert("writeLine".to_string(), native("Console.writeLine", 1));
    console_fields.insert("clear".to_string(), native("Console.clear", 1));
    vm.globals.insert(
        "Console".to_string(),
        Value::Record(Arc::new(Mutex::new(console_fields))),
    );

    // Script Module
    let mut script_fields = HashMap::new();
    script_fields.insert("eval".to_string(), native("Script.eval", 1));
    script_fields.insert("evalToString".to_string(), native("Script.evalToString", 1));
    vm.globals.insert(
        "Script".to_string(),
        Value::Record(Arc::new(Mutex::new(script_fields))),
    );

    // Async Module
    let mut async_fields = HashMap::new();
    async_fields.insert("Return".to_string(), native("Async.Return", 1));
    async_fields.insert("ReturnFrom".to_string(), native("Async.ReturnFrom", 1));
    async_fields.insert("Bind".to_string(), native("Async.Bind", 2));
    async_fields.insert("Delay".to_string(), native("Async.Delay", 1));
    async_fields.insert("Zero".to_string(), native("Async.Zero", 0));
    async_fields.insert("Combine".to_string(), native("Async.Combine", 2));
    async_fields.insert("RunSynchronously".to_string(), native("Async.RunSynchronously", 1));
    vm.globals.insert(
        "Async".to_string(),
        Value::Record(Arc::new(Mutex::new(async_fields))),
    );

    // Register 'async' builder alias (lowercase) to point to Async module
    // This is required because 'async { ... }' desugars to 'async.Bind', etc.
    if let Some(async_val) = vm.globals.get("Async") {
        vm.globals.insert("async".to_string(), async_val.clone());
    }
}

fn wrap_unary<F>(args: &[Value], f: F) -> Result<Value, VmError>
where
    F: Fn(&Value) -> Result<Value, VmError>,
{
    if args.len() != 1 {
        return Err(VmError::Runtime(format!(
            "Expected 1 argument, got {}",
            args.len()
        )));
    }
    f(&args[0])
}

fn wrap_binary<F>(args: &[Value], f: F) -> Result<Value, VmError>
where
    F: Fn(&Value, &Value) -> Result<Value, VmError>,
{
    if args.len() != 2 {
        return Err(VmError::Runtime(format!(
            "Expected 2 arguments, got {}",
            args.len()
        )));
    }
    f(&args[0], &args[1])
}

fn wrap_ternary<F>(args: &[Value], f: F) -> Result<Value, VmError>
where
    F: Fn(&Value, &Value, &Value) -> Result<Value, VmError>,
{
    if args.len() != 3 {
        return Err(VmError::Runtime(format!(
            "Expected 3 arguments, got {}",
            args.len()
        )));
    }
    f(&args[0], &args[1], &args[2])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_stdlib() {
        let mut vm = Vm::new();
        register_stdlib(&mut vm);

        // Check HostRegistry
        assert!(vm.host_registry.lock().unwrap().has_function("List.length"));

        // Check Globals
        assert!(vm.globals.contains_key("List"));
        if let Some(Value::Record(r)) = vm.globals.get("List") {
            assert!(r.lock().unwrap().contains_key("length"));
        } else {
            panic!("List global is not a record");
        }
    }

    #[test]
    fn test_register_option_functions() {
        let mut vm = Vm::new();
        register_stdlib(&mut vm);

        // Check all Option functions are registered
        assert!(vm.host_registry.lock().unwrap().has_function("Option.isSome"));
        assert!(vm.host_registry.lock().unwrap().has_function("Option.isNone"));
        assert!(vm.host_registry.lock().unwrap().has_function("Option.defaultValue"));
        assert!(vm.host_registry.lock().unwrap().has_function("Option.defaultWith"));
        assert!(vm.host_registry.lock().unwrap().has_function("Option.map"));
        assert!(vm.host_registry.lock().unwrap().has_function("Option.bind"));
        assert!(vm.host_registry.lock().unwrap().has_function("Option.iter"));
        assert!(vm.host_registry.lock().unwrap().has_function("Option.map2"));
        assert!(vm.host_registry.lock().unwrap().has_function("Option.orElse"));

        // Check constructors
        assert!(vm.host_registry.lock().unwrap().has_function("Some"));
        assert!(vm.host_registry.lock().unwrap().has_function("None"));
    }

    #[test]
    fn test_register_option_globals() {
        let mut vm = Vm::new();
        register_stdlib(&mut vm);

        // Check Option module global
        assert!(vm.globals.contains_key("Option"));
        if let Some(Value::Record(r)) = vm.globals.get("Option") {
            let borrowed = r.lock().unwrap();
            assert!(borrowed.contains_key("isSome"));
            assert!(borrowed.contains_key("isNone"));
            assert!(borrowed.contains_key("defaultValue"));
            assert!(borrowed.contains_key("defaultWith"));
            assert!(borrowed.contains_key("map"));
            assert!(borrowed.contains_key("bind"));
            assert!(borrowed.contains_key("iter"));
            assert!(borrowed.contains_key("map2"));
            assert!(borrowed.contains_key("orElse"));
        } else {
            panic!("Option global is not a record");
        }

        // Check constructor globals
        assert!(vm.globals.contains_key("Some"));
        assert!(vm.globals.contains_key("None"));
    }
}
