/**
 * Squirrel Standard Library Signatures
 * Generated from https://wiki.teamfortress.com/wiki/Team_Fortress_2/Scripting/Script_Functions
 * Only for reference, do not modify
 * @native
 */

const _charsize_ = 1;
const _floatsize_ = 4;

/**
 * 32-bit: `4`
 * 64-bit: `8`
 */
const _intsize_ = 8;

const _version_ = "Squirrel 3.2 stable";
const _versionnumber_ = 320;

/**
 * Windows: `32767`
 * Linux: `2147483647`
 */
const RAND_MAX = 2147483647

const PI = 3.14159

/**
 * Throws an assertion error if the given expression evaluates to false (i.e. the values 0, 0.0, -0.0, null and false).
 * @param {any} exp
 * @throws {string} if the given expression evaluates to false
 */
function assert(exp);

/**
 * Interprets the float's bytes as if it were a 32-bit integer representation.
 * @param {float} value
 * @returns {integer}
 */
function castf2i(value);

/**
 * Interprets the integer's bytes as if it were a floating-point encoding.
 * @param {integer} value
 * @returns {float}
 */
function casti2f(value);

/**
 * Runs the garbage collector and returns the number of reference cycles found and deleted.
 * @returns {integer}
 */
function collectgarbage();

/**
 * Compiles a string containing a squirrel script into a function and returns it.
 * @param {string} code
 * @param {string|null} buffer_name
 * @returns {function}
 */
function compilestring(code, buffer_name = null);

/**
 * Returns and does nothing. Can be used as an empty function placeholder.
 * @varargs {any}
 */
function dummy(...);

/**
 * Enable or disable debug line information generation at compile time.
 * @param {bool} enable
 */
function enabledebuginfo(enable);

/**
 * Prints message to the standard error output.
 * @param {any} message
 */
function error(message);

/**
 * Returns the const table of the VM.
 * @returns {table}
 */
function getconsttable();

/**
 * Returns the root table of the VM.
 * @returns {table}
 */
function getroottable();

/**
 * Returns the stack frame information at the given stack level. 0 is the current function, 1 is the caller and so on. Returns null if the stack level doesn't exist.
 * @param {integer} level
 * @returns {table|null}
 */
function getstackinfos(level);

/**
 * Prints the given parameter with no newline.
 * @param {any} message
 */
function print(message);

/**
 * Runs the garbage collector and returns an array containing all unreachable objects found. Returns null if none are found.
 * @returns {array}
 */
function resurrectunreachable();

/**
 * Sets the const table of the VM and returns the previous const table.
 * @param {table} const_table
 * @returns {table}
 */
function setconsttable(const_table);

/**
 * Sets the debug hook.
 * @param {function} hook_func
 */
function setdebughook(hook_func);

/**
 * Sets the runtime error handler.
 * @param {function|null} error_func
 */
function seterrorhandler(error_func);

/**
 * Sets the root table of the VM and returns the previous root table.
 * @param {table} table
 * @returns {table}
 */
function setroottable(table);

/**
 * Swaps bytes 1 and 2 of the integer.
 * @param {integer} value
 * @returns {integer}
 */
function swap2(value);

/**
 * Reverses the byte order of the four bytes of an integer.
 * @param {integer} value
 * @returns {integer}
 */
function swap4(value);

/**
 * Reverses the byte order of the four bytes of a float.
 * @param {float} value
 * @returns {float}
 */
function swapfloat(value);

/**
 * Returns the native type of the given parameter as a string, bypassing the _typeof delegate method.
 * @param {any} value
 * @returns {string}
 */
function type(value);

/**
 * Returns |x| as integer.
 * @param {float} x
 * @returns {integer}
 */
function abs(x);

/**
 * Returns the arc cosine of x in radians.
 * @param {float} x
 * @returns {float}
 */
function acos(x);

/**
 * Returns the arc sine of x in radians.
 * @param {float} x
 * @returns {float}
 */
function asin(x);

/**
 * Returns the arc tangent of x in radians.
 * @param {float} x
 * @returns {float}
 */
function atan(x);

/**
 * Returns the angle between the ray from (0,0) through (x,y) and the positive x-axis, in the range (-PI, PI].
 * @param {float} y
 * @param {float} x
 * @returns {float}
 */
function atan2(y, x);

/**
 * Returns the smallest integer that is >= x as a float.
 * @param {float} x
 * @returns {float}
 */
function ceil(x);

/**
 * Returns the cosine of x.
 * @param {float} x
 * @returns {float}
 */
function cos(x);

/**
 * Returns e raised to the power of x.
 * @param {float} x
 * @returns {float}
 */
function exp(x);

/**
 * Returns |x| as float.
 * @param {float} x
 * @returns {float}
 */
function fabs(x);

/**
 * Returns the largest integer that is <= x as a float.
 * @param {float} x
 * @returns {float}
 */
function floor(x);

/**
 * Returns the natural logarithm of x.
 * @param {float} x
 * @returns {float}
 */
function log(x);

/**
 * Returns the base-10 logarithm of x.
 * @param {float} x
 * @returns {float}
 */
function log10(x);

/**
 * Returns `x` raised to the power of `y`.
 * @param {float} x
 * @param {float} y
 * @returns {float}
 */
function pow(x, y);

/**
 * Returns a random integer where `0 <= rand() <= RAND_MAX`.
 * @returns {integer}
 */
function rand();

/**
 * Returns the sine of value.
 * @param {float} value
 * @returns {float}
 */
function sin(value);

/**
 * Returns the square root of value.
 * @param {float} value
 * @returns {float}
 */
function sqrt(value);

/**
 * Sets the starting point for generating a series of pseudorandom integers.
 * @param {integer} seed
 */
function srand(seed);

/**
 * Returns the tangent of x.
 * @param {float} x
 * @returns {float}
 */
function tan(x);

/**
 * Returns true if str ends with cmp.
 * @param {string} str
 * @param {string} cmp
 * @returns {bool}
 */
function endswith(str, cmp);

/**
 * Returns a string with backslashes inserted before characters that need to be escaped.
 * @param {string} str
 * @returns {string}
 */
function escape(str);

/**
 * Returns a formatted string using printf-style format specifiers.
 * @param {string} str
 * @varargs {any}
 * @returns {string}
 */
function format(str, ...);

/**
 * Removes whitespace from the beginning of the string.
 * @param {string} str
 * @returns {string}
 */
function lstrip(str);

/**
 * Removes whitespace from the end of the string.
 * @param {string} str
 * @returns {string}
 */
function rstrip(str);

/**
 * Returns an array of strings split at each occurrence of a separator character.
 * @param {string} str
 * @param {string} separator
 * @param {bool} skip_empty
 * @returns {array}
 */
function split(str, separator, skip_empty = false);

/**
 * Returns true if str starts with cmp.
 * @param {string} str
 * @param {string} cmp
 * @returns {bool}
 */
function startswith(str, cmp);

/**
 * Removes whitespace from both the beginning and end of the string.
 * @param {string} str
 * @returns {string}
 */
function strip(str);

/**
 * Returns a new array of the given length where each element is set to fill.
 * @param {integer} length
 * @param {any} fill
 * @returns {array}
 */
function array(length, fill = null);

/**
 * Creates a new cooperative thread object and returns it.
 * @param {function} func
 * @returns {thread}
 */
function newthread(func);

/**
 * Suspends the coroutine that called this function.
 * @param {any} return_value
 * @returns {any}
 */
function suspend(return_value = this);

class regexp {
    /**
     * Creates and compiles a regular expression from the given pattern.
     * @param {string} pattern
     */
    constructor(pattern);

    /**
     * Returns an array of tables with "begin" and "end" keys for the first match and each captured sub-expression. Returns null if no match occurs.
     * @param {string} str
     * @param {integer} start
     * @returns {table|null}
     */
    function capture(str, start = 0);

    /**
     * Returns true if the regular expression matches the entire string.
     * @param {string} str
     * @returns {bool}
     */
    function match(str);

    /**
     * Returns a table with "begin" and "end" keys for the first match in str, or null if no match occurs.
     * @param {string} str
     * @param {integer} start
     * @returns {table|null}
     */
    function search(str, start = 0);

    /**
     * Returns the number of sub-expression groups in the regular expression. Always >= 1 since the whole regex counts as a group.
     * @returns {integer}
     */
    function subexpcount();
}

class blob {
    /**
     * Creates a new blob of the given initial size.
     * @param {integer} init_size
     */
    constructor(init_size = 0);

    /**
     * Returns non-zero if the current read/write position is at the end of the blob.
     * @returns {integer}
     */
    function eos();

    /**
     * Flushes the blob stream.
     */
    function flush();

    /**
     * Returns the size of the blob in bytes.
     * @returns {integer}
     */
    function len();

    /**
     * Reads the specified number of bytes and returns them as a new blob.
     * @param {integer} num_of_bytes
     * @returns {blob}
     */
    function readblob(num_of_bytes);

    /**
     * Reads a number from the blob according to the data type character.
     * @param {integer} data_type
     * @returns {any}
     */
    function readn(data_type);

    /**
     * Resizes the blob to the specified size.
     * @param {integer} new_size
     */
    function resize(new_size);

    /**
     * Moves the read/write position. Returns 0 on success.
     * @param {integer} offset
     * @param {integer} offset_basis
     * @returns {integer}
     */
    function seek(offset, offset_basis);

    /**
     * Swaps the byte order of all 2-byte aligned values in the blob.
     */
    function swap2();

    /**
     * Swaps the byte order of all 4-byte aligned values in the blob.
     */
    function swap4();

    /**
     * Returns the current read/write position.
     * @returns {integer}
     */
    function tell();

    /**
     * Writes the contents of another blob into this blob at the current position.
     * @param {blob} src
     */
    function writeblob(src);

    /**
     * Writes a number to the blob according to the data type character.
     * @param {any} value
     * @param {integer} data_type
     */
    function writen(value, data_type);

    /**
     * Writes a string into the blob at the current position.
     * @param {string} str
     */
    function writestring(str);
}