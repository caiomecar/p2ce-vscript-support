function AddThinkToEnt(entity, think_name);

class Vector {
    x = null
    y = null
    z = null

    constructor(x = 0.0, y = 0.0, z = 0.0);

    /**
     * @param {Vector} other
     */
    function _add(other);

    /**
     * @param {float} other
     */
    function _mul(other);

    function Cross(factor);

    function Dot(factor);

    function Length();

    function LengthSqr();

    function Length2D();

    function Length2DSqr();

    function Norm();

    function Scale(factor);

    function ToKVString();

    function tostring();
}