class integer {
    function tofloat();

    function tostring();

    function tochar();
}

class float {
    function tointeger();

    function tostring();

    function tochar();
}

class bool {
    function tofloat();

    function tointeger();

    function tostring();
}

class string {
    function find(search_string, start_index = 0);

    function len();

    function slice(start_index, end_index = -1);

    function tofloat();

    function tointeger();

    function tolower();

    function toupper();
}

class array {
    function append(item);

    function apply(item);

    function clear();

    function extend(other);

    function filter(condition);

    function find(element);

    function insert(index, item);

    function len();

    function map(func);

    function pop();

    function push(item);

    // A lil bit chilly innit
    function reduce(func, init = null);

    function remove(index);

    function resize(new_size, fill = null);

    function reverse();

    /**
     * @param {integer} start_index
     */
    function slice(start_index, end_index = -1);

    function sort(compare = @(a, b) a <=> b);

    function top();

    function tostring();

    function weakref();
}

class table {
    function clear();

    /**
     * @returns {table}
     */
    function filter(func);

    /**
     * @returns {table}
     */
    function getdelegate();

    /**
     * @returns {array}
     */
    function keys();

    /**
     * @returns {integer}
     */
    function len();

    /**
     * @returns {any}
     */
    function rawdelete(key);

    /**
     * @returns {any}
     */
    function rawget(key);

    /**
     * @returns {any}
     */
    function rawin(key);

    /**
     * @returns {table}
     */
    function rawset(key);

    /**
     * @returns {table}
     */
    function setdelegate(delegate);

    /**
     * @returns {array}
     */
    function values();

    /**
     * @returns {string}
     */
    function tostring();
}

class function_ {
    function acall(args);

    function bindenv(env);

    function call(env, ...);

    function getinfos();

    function getroot();

    function pacall(args);

    function pcall(env, args);

    function setroot(root);

    function tostring();

    function weakref();
}

class class_ {
    function getattributes(member_name);

    function instance();

    function newmember(key, value, attributes = {}, is_static = false);

    function rawdelete(key);

    function rawget(key);

    function rawin(key);

    function rawnewmember(key, value, attributes = {}, is_static = false);

    function rawset(key);

    function setattributes(name, value);

    function tostring();
}

class instance {
    function getclass();

    function rawget(key);

    function rawin(key);

    function rawset(key);

    function tostring();

    function weakref();
}

class generator {
    function getstatus();

    function tostring();

    function weakref();
}

class thread {
    function call(...);

    function getstackinfos(level);

    function getstatus();

    function tostring();

    function wakeup(return_value = null);

    function wakeupthrow(obj_to_throw, propagate_error = true);

    function weakref();
}

class weakref {
    function ref();

    function tostring();
}