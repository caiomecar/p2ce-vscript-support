/**
 * Squirrel Builtins Signatures
 * Generated from https://wiki.teamfortress.com/wiki/Team_Fortress_2/Scripting/Script_Functions
 * Only for reference, do not modify
 * @native
 */

class integer {
    /**
     * Converts the integer to float and returns it.
     * @returns {float}
     */
    function tofloat();

    /**
     * Converts the integer to string and returns it.
     * @returns {string}
     */
    function tostring();

    /**
     * Returns a string containing a single character represented by the integer.
     * Only works for ascii values (0-127 range), otherwise returns "?" if integer is outside of this range.
     * @returns {string}
     */
    function tochar();

    /**
     * Returns the integer itself
     * @returns {integer}
     * @hide
     */
    function tointeger();

    /**
     * Returns the integer itself
     * @returns {integer}
     * @hide
     */
    function weakref();
}

class float {
    /**
     * Converts the float to integer and returns it. Returns INT_MIN for inf, -inf and NaN
     * @returns {integer}
     */
    function tointeger();

    /**
     * Converts the float to string and returns it.
     * @returns {string}
     */
    function tostring();

    /**
     * Returns a string containing a single character represented by the integer part of the float.
     * Only works for ascii values (0-127 range), otherwise returns "?" if float is outside of this range.
     * @returns {string}
     */
    function tochar();

    /**
     * Returns the float itself
     * @returns {float}
     * @hide
     */
    function weakref();
}

class bool {
    /**
     * Returns 1.0 for true, 0.0 for false.
     * @returns {float}
     */
    function tofloat();

    /**
     * Returns 1 for true, 0 for false.
     * @returns {integer}
     */
    function tointeger();

    /**
     * Returns "true" for true and "false" for false.
     * @returns {string}
     */
    function tostring();

    /**
     * Returns the bool itself
     * @returns {bool}
     * @hide
     */
    function weakref();
}

class string {
    /**
     * Looks for the sub-string passed as its first parameter,starting at either the beginning
     * of the string or at a specific character index if one is provided as a second parameter.
     * If the sub-string is found, returns the index at which it first occurs, otherwise returns null.
     * @param {string} search_string
     * @param {integer} start_index
     * @returns {integer|null}
     */
    function find(search_string, start_index = 0);

    /**
     * Returns the length of the string, ie. the number of characters it comprises.
     * @returns {integer}
     */
    function len();

    /**
     * Creates a sub-string from a string. Copies characters from start_index to end_index.
     * The sub-string includes the character at start_index, but excludes the one at end_index.
     * If end_index is not specified, copies until the last character.
     * If the provided start or end index is beyond the string, an exception is thrown.
     * If the numbers are negative the count will start from the end of the string
     * (e.g. -2 represents a second last character).
     * @param {integer} start_index
     * @param {integer} end_index
     * @returns {integer}
     */
    function slice(start_index, end_index = -1);

    /**
     * Returns float value represented by the string. Must only contain numeric characters
     * and/or plus and minus symbols. An exception is thrown otherwise.
     * @returns {float}
     * @throws {string}
     */
    function tofloat();

    /**
     * Returns integer value represented by the string. Must only contain numeric characters.
     * An exception is thrown otherwise. Hexadecimal notation is supported (i.e. 0xFF).
     * If a hexadecimal string contains more than 10 characters, including the 0x, returns -1.
     * @param {integer} number_base
     * @returns {integer}
     * @throws {string}
     */
    function tointeger(number_base = 10);

    /**
     * Returns a new string with all upper-case characters converted to lower-case.
     * @returns {string}
     */
    function tolower();

    /**
     * Returns a new string with all lower-case characters converted to upper-case.
     * @returns {string}
     */
    function toupper();

    /**
     * Returns a weak reference pointing to the string
     * @returns {weakref}
     */
    function weakref();

    /**
     * Returns the string itself
     * @returns {string}
     * @hide
     */
    function tostring();
}

class array {
    /**
     * Returns a new array of the given length where each element is set to fill.
     * Can also be created with an array literal, e.g. [1, 2, 3].
     * @param {integer} length
     * @param {any} fill
     */
    constructor(length, fill = null);

    /**
     * Adds an item to the end of the array.
     * @param {any} item
     */
    function append(item);

    /**
     * Applies a function to all of the array's items and replaces the original value of each
     * element with the return value of the function.
     * The provided func can accept up to 3 arguments: array item value (required),
     * array item index (optional), reference to the array itself (optional).
     * @param {function} func
     */
    function apply(func);

    /**
     * Removes all of the items from the array.
     */
    function clear();

    /**
     * Combines two arrays into one.
     * @param {array} other
     * @returns {array}
     */
    function extend(other);

    /**
     * Applies a filter function to the array's items, storing the results in a new array.
     * @param {function} condition - function(int index, any value) : bool
     * @returns {array}
     */
    function filter(condition);

    /**
     * Looks for the element passed as its parameter, starting at the beginning of the array.
     * If the element is found, returns the index at which it first occurs, otherwise returns null.
     * @param {any} element
     * @returns {integer|null}
     */
    function find(element);

    /**
     * Inserts an item into the array at the specified index.
     * @param {integer} index
     * @param {any} item
     */
    function insert(index, item);

    /**
     * Returns the length of the array, ie. the number of elements it has.
     * @returns {integer}
     */
    function len();

    /**
     * Creates a new array of the same size. For each element in the original array invokes
     * the function func and assigns the return value to the corresponding element of the new array.
     * The provided func can accept up to 3 arguments: array item value (required),
     * array item index (optional), reference to the array itself (optional).
     * @param {function} func
     * @returns {array}
     */
    function map(func);

    /**
     * Returns and removes the value at the end of the array.
     * Throws an exception if the array is empty.
     * @returns {any}
     * @throws {string}
     */
    function pop();

    /**
     * Adds an item to the end of the array.
     * @param {any} item
     */
    function push(item);

    /**
     * Applies the supplied function to all of the items in the array, starting with the first two.
     * The function returns a single value which is then combined with the next item — and so on
     * until all items have been combined into a single value which the method returns.
     * @param {function} func - function(pre_value: any, current_value: any) -> any
     * @param {any} init
     * @returns {any}
     */
    function reduce(func, init = null);

    /**
     * Returns and removes an array item at the specified index.
     * Throws an exception if the index is outside the array's boundaries.
     * @param {integer} index
     * @returns {any}
     * @throws {string}
     */
    function remove(index);

    /**
     * Increases or decreases the size of the array.
     * In case of increasing, fills the new spots with the fill parameter.
     * @param {integer} new_size
     * @param {any} fill
     */
    function resize(new_size, fill = null);

    /**
     * Reverses the order of the elements in the array.
     */
    function reverse();

    /**
     * Creates a new array from the array. Copies elements from start_index to end_index.
     * The new array includes the element at start_index, but excludes the one at end_index.
     * If end_index is not specified, copies until the last element.
     * If the provided start or end index is beyond the array, an exception is thrown.
     * If the numbers are negative the count will start from the end of the array
     * (e.g. -2 represents a second last element).
     * @param {integer} start_index
     * @param {integer} end_index
     * @returns {array}
     * @throws {string}
     */
    function slice(start_index, end_index = -1);

    /**
     * Sorts the items within the array into lowest-to-highest order, or according to the
     * results of an optional comparison function. If items are arrays, blobs, functions,
     * objects and/or tables, they will be sorted by reference not value.
     * The comparison function should take two parameters and return -1 if the first value
     * should be placed before the second, 1 if it should follow, or 0 if they are equivalent.
     * The spaceship operator <=> may come in handy, e.g. arr.sort(\@(a, b) a.distance <=> b.distance).
     * @param {function} compare - function(a: any, b: any) -> integer
     */
    function sort(compare = @(a, b) a <=> b);

    /**
     * Returns the value at the end of the array.
     * Throws an exception if the array is empty.
     * @returns {any}
     * @throws {string}
     */
    function top();

    /**
     * Returns the string "(array : pointer)".
     * @returns {string}
     */
    function tostring();

    /**
     * Returns a weak reference pointing to the array
     * @returns {weakref}
     */
    function weakref();
}

class table {
    /**
     * Removes all of the items from the table.
     */
    function clear();

    /**
     * Creates a new table with all values that pass the test implemented by the provided function.
     * Invokes the function for each key-value pair; if it returns true, the value is added
     * to the new table at the same key.
     * @param {function} func - function(key: any, value: any) -> bool
     * @returns {table}
     */
    function filter(func);

    /**
     * Returns the table's delegate.
     * @returns {table|null}
     */
    function getdelegate();

    /**
     * Returns an array containing all the keys of the table slots.
     * @returns {array}
     */
    function keys();

    /**
     * Returns the length of the table, ie. the number of entries it has.
     * @returns {integer}
     */
    function len();

    /**
     * Deletes the target slot without employing delegation.
     * If the table lacks the target slot, returns null, otherwise returns the associated value.
     * @param {any} key
     * @returns {any}
     */
    function rawdelete(key);

    /**
     * Retrieves the value of the specified key without employing delegation.
     * @param {any} key
     * @returns {any}
     */
    function rawget(key);

    /**
     * Checks for the presence of the specified key in the table without employing delegation.
     * @param {any} key
     * @returns {bool}
     */
    function rawin(key);

    /**
     * Sets the value of the specified key without employing delegation. Returns the table itself.
     * @param {any} key
     * @param {any} value
     * @returns {table}
     */
    function rawset(key, value);

    /**
     * Assigns the passed table as the target's new custom delegate. Returns the target table.
     * To remove a delegate, pass null.
     * @param {table|null} delegate
     * @returns {table}
     */
    function setdelegate(delegate);

    /**
     * Returns an array containing all the values of the table slots.
     * @returns {array}
     */
    function values();

    /**
     * Tries to invoke the _tostring metamethod. If that fails returns the string "(table: pointer)".
     * @returns {string}
     */
    function tostring();

    /**
     * Returns a weak reference pointing to the table
     * @returns {weakref}
     */
    function weakref();
}

class function_ {
    /**
     * Calls the target function and passes array values into its parameters.
     * First element of the array should be the non-default context object.
     * @param {array} args
     * @returns {any}
     */
    function acall(args);

    /**
     * Clones the target function and binds it to a specified context object.
     * @param {table|instance|null} environment
     * @returns {function}
     */
    function bindenv(environment);

    /**
     * Calls the function with a non-default context object.
     * @param {table|instance|null} environment
     * @varargs {any}
     * @returns {any}
     */
    function call(environment, ...);

    /**
     * Returns a table containing information about the function,
     * such as parameters, name and source name.
     * @returns {table}
     */
    function getinfos();

    /**
     * Returns the root table of the closure.
     * @returns {table}
     */
    function getroot();

    /**
     * Calls the function with an array of parameters, bypassing Squirrel error callbacks.
     * First element of the array should be the non-default context object.
     * @param {array} args
     * @returns {any}
     */
    function pacall(args);

    /**
     * Calls the function with a non-default context object, bypassing Squirrel error callbacks.
     * @param {table|instance|null} environment
     * @varargs {any}
     * @returns {any}
     */
    function pcall(environment, ...);

    /**
     * Sets the root table of the closure.
     * @param {table} root
     */
    function setroot(root);

    /**
     * Returns the string "(closure: pointer)".
     * @returns {string}
     */
    function tostring();
}

class class_ {
    /**
     * Returns the attributes of the specified member.
     * If member_name is null, returns the class-level attributes.
     * @param {string|null} member_name
     * @returns {any}
     */
    function getattributes(member_name);

    /**
     * Returns a new instance of the class. Does not invoke the instance constructor.
     * The constructor must be explicitly called (e.g. class_inst.constructor(class_inst)).
     * @returns {instance}
     */
    function instance();

    /**
     * Sets/adds the slot key with the value and attributes, and if present invokes
     * the _newmember metamethod. If static is true, the slot will be added as static.
     * If the slot does not exist, it will be created.
     * @param {any} key
     * @param {any} value
     * @param {table} attributes
     * @param {bool} is_static
     */
    function newmember(key, value, attributes = {}, is_static = false);

    /**
     * Deletes the target slot without employing delegation.
     * Returns null if the slot is missing, otherwise returns the associated value.
     * @param {any} key
     * @returns {any}
     */
    function rawdelete(key);

    /**
     * Retrieves the value of the specified key without employing delegation.
     * @param {any} key
     * @returns {any}
     */
    function rawget(key);

    /**
     * Checks for the presence of the specified key in the class without employing delegation.
     * @param {any} key
     * @returns {bool}
     */
    function rawin(key);

    /**
     * Sets/adds the slot key with the value and attributes, without invoking _newmember.
     * If static is true, the slot will be added as static.
     * If the slot does not exist, it will be created.
     * @param {any} key
     * @param {any} value
     * @param {table} attributes
     * @param {bool} is_static
     */
    function rawnewmember(key, value, attributes = {}, is_static = false);

    /**
     * Sets the value of the specified key without employing delegation. Returns the class itself.
     * @param {any} key
     * @param {any} value
     * @returns {class}
     */
    function rawset(key, value);

    /**
     * Sets the attribute of the specified member and returns the previous attribute value.
     * If member_name is null, sets the class-level attributes.
     * @param {string|null} member_name
     * @param {any} value
     * @returns {any}
     */
    function setattributes(member_name, value);

    /**
     * Returns the string "(class: pointer)".
     * @returns {string}
     */
    function tostring();

    /**
     * Returns a weak reference pointing to the class
     * @returns {weakref}
     */
    function weakref();
}

class instance {
    /**
     * Returns the class that created the instance.
     * @returns {class}
     */
    function getclass();

    /**
     * Retrieves the value of the specified key without employing delegation.
     * @param {any} key
     * @returns {any}
     */
    function rawget(key);

    /**
     * Checks for the presence of the specified key in the instance without employing delegation.
     * @param {any} key
     * @returns {bool}
     */
    function rawin(key);

    /**
     * Sets the value of the specified key without employing delegation. Returns the instance itself.
     * @param {any} key
     * @param {any} value
     * @returns {instance}
     */
    function rawset(key, value);

    /**
     * Tries to invoke the _tostring metamethod. If that fails returns the string "(instance: pointer)".
     * @returns {string}
     */
    function tostring();

    /**
     * Returns a weak reference pointing to the instance
     * @returns {weakref}
     */
    function weakref();
}

class generator {
    /**
     * Returns the status of the generator as a string: "running", "dead" or "suspended".
     * @returns {string}
     */
    function getstatus();

    /**
     * Returns the string "(generator : pointer)".
     * @returns {string}
     */
    function tostring();

    /**
     * Returns a weak reference pointing to the generator
     * @returns {weakref}
     */
    function weakref();
}

class thread {
    /**
     * Starts the thread with the specified parameters. Returns either the first suspend value
     * or the returned value of the function if no suspends were triggered.
     * @varargs {any}
     * @returns {any}
     */
    function call(...);

    /**
     * Returns the stack frame information at the given stack level.
     * (0 is the current function, 1 is the caller, and so on.)
     * Returns null if the stack level doesn't exist.
     * @param {integer} level
     * @returns {table|null}
     */
    function getstackinfos(level);

    /**
     * Returns the status of the thread as a string: "idle", "running" or "suspended".
     * @returns {string}
     */
    function getstatus();

    /**
     * Returns the string "(thread : pointer)".
     * @returns {string}
     */
    function tostring();

    /**
     * Returns a weak reference pointing to the thread
     * @returns {weakref}
     */
    function weakref();

    /**
     * Wakes up a suspended thread. The optional return_value will be used as the return value
     * for the suspend() call that paused the thread. Returns the next suspended value or
     * the thread's return value.
     * @param {any} return_value
     * @returns {any}
     */
    function wakeup(return_value = null);

    /**
     * Wakes up a suspended thread, throwing obj_to_throw as an exception inside it.
     * @param {any} obj_to_throw
     * @param {bool} propagate_error
     * @returns {any}
     */
    function wakeupthrow(obj_to_throw, propagate_error = true);
}

class weakref {
    /**
     * Returns the object that the weak reference is pointing at.
     * Returns null if the referenced object has been destroyed.
     * @returns {instance}
     */
    function ref();

    /**
     * Returns the string "(weakref : pointer)".
     * @returns {string}
     */
    function tostring();

    /**
     * Returns a weak reference pointing to the weakref
     * @returns {weakref}
     */
    function weakref();
}