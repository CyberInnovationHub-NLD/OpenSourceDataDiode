# -*- coding: utf-8 -*-
###############################################################################
## (C) COPYRIGHT 2018 TECHNOLUTION BV, GOUDA NL
## | =======          I                   ==          I    =
## |    I             I                    I          I
## |    I   ===   === I ===  I ===   ===   I  I    I ====  I   ===  I ===
## |    I  /   \ I    I/   I I/   I I   I  I  I    I  I    I  I   I I/   I
## |    I  ===== I    I    I I    I I   I  I  I    I  I    I  I   I I    I
## |    I  \     I    I    I I    I I   I  I  I   /I  \    I  I   I I    I
## |    I   ===   === I    I I    I  ===  ===  === I   ==  I   ===  I    I
## |                 +---------------------------------------------------+
## +----+            |  +++++++++++++++++++++++++++++++++++++++++++++++++|
##      |            |             ++++++++++++++++++++++++++++++++++++++|
##      +------------+                          +++++++++++++++++++++++++|
##                                                         ++++++++++++++|
##                                                                  +++++|
###############################################################################
""" Importer class to import Python modules in Technolution-style projects

    This importer is loaded into the python module loader system upon loading.
    While loading, it parses the $TL_IMPORT_LIBS environment variable, which
    must exist, and must adhere to the following format:

        entry := python_name "=" path_to_directory
        tl_import_libs := entry ( ":" entry )

    For convenience, it is permitted for entries to be empty, and whitespace
    between the "=" and ":" tokens is ignored. Entries can be overridden by
    appending an entry with the same name.

    Usually, $TL_IMPORT_LIBS will look like this:

        <prjname>=<prjroot>:ip=<prjroot>/ip

    As a bare minimum for using IP's BFMs, $TL_IMPORT_LIBS must include the
    root directory of the IP library for the entry "ip".

    The importer class then looks for import statements that start with one
    of the python_names in $TL_IMPORT_LIBS and uses it to resolve the root
    directory for the module path. The rest of the module path corresponds to
    the directory tree using the following rules in the following order:

     - If the root_dir + module_path map to a directory with an __init__.py
       file inside, the __init__.py module is imported.
     - If the root_dir + module_path map to a directory WITHOUT an __init__.py
       file inside, a virtual module is generated. These virtual modules
       emulate an __init__.py file that pulls all its submodules except those
       starting with an underscore into its own namespace, although it is lazy
       about actually loading the modules. The practical upshot of these
       virtual modules is that you don't need to spam empty __init__.py files
       everywhere just to get Python to traverse the directory tree.
     - If the root_dir + module_path + ".py" exists, this file is loaded.
     - If the root_dir + module_path[:-1] + "/py_*/ + module_path[-1] + ".py"
       exists -- that is, the referred Python file is within a py_*
       subdirectory -- this file is loaded. If this rule matches multiple
       files, the resolution is alphabetical (for example, py_bfm is scanned
       before py_sim). Disambiguation is possible by explicitly specifying the
       path, but if you think you need this, you should probably think again.

    For example, ccart can be loaded as follows:

        import ip.core.simulation.cc_utils.ccart as ccart
        ccart.CoCoARTTestCase

    But also like this:

        import ip.core.simulation.cc_utils as cc_utils
        cc_utils.ccart.CoCoARTTestCase
        cc_utils.cc_queue.CcQueue

    Or:

        from ip.core.simulation.cc_utils import ccart
        ccart.CoCoARTTestCase

    Or:

        from ip.core.simulation.cc_utils import *
        ccart.CoCoARTTestCase
        cc_queue.CcQueue

    Or, if you're masochistic:

        import ip
        ip.core.simulation.cc_utils.py_bfm.ccart.CoCoARTTestCase
        ip.core.simulation.cc_utils.py_bfm.cc_queue.CcQueue

    The same patterns apply for any other entry in $TL_IMPORT_LIBS. As a silly
    example, if your project is named "tl" and the IP submodule is in the root
    of your project, this would also work:

        import tl.ip.core.simulation.cc_utils.ccart as ccart
        ccart.CoCoARTTestCase

    @author     : Jeroen van Straten (jeroen.van.straten@technolution.nl)
"""

import imp
import sys
import os
import logging
import types
import re

logger = logging.getLogger("tl_import")


class TlImportError(ImportError):
    """Can be used to distinguish between an ImportError from tl_import and
    Python's own ImportErrors."""
    pass


class TlVirtualModule(types.ModuleType):
    """Represents a module for a nonexistent Python file."""

    def __init__(self, loader, fullname, path):
        name = fullname.split(".")[-1]
        super().__init__(name)
        self.__loader__ = loader
        self.__file__ = "[virtual module \"%s\"]" % fullname
        self.__path__ = []
        self._fullname = fullname
        self._path = os.path.dirname(path)

        # Figure out which Python files and submodules exist on our path and
        # set it to __all__ so "from x import *" works as intended.
        sub = set()
        for filename in os.listdir(self._path):
            path = os.path.join(self._path, filename)
            name, ext = os.path.splitext(filename)
            if not re.match(r"[a-zA-Z][a-zA-Z0-9_]*", name):
                continue
            if os.path.isdir(path):
                sub.add(name)
            if ext == ".py" and os.path.isfile(path):
                sub.add(name)
            subdir = filename
            subpath = os.path.join(self._path, subdir)
            if subdir.startswith("py_") and os.path.isdir(subpath):
                for filename in os.listdir(subpath):
                    path = os.path.join(subpath, filename)
                    name, ext = os.path.splitext(filename)
                    if not re.match(r"[a-zA-Z][a-zA-Z0-9_]*", name):
                        continue
                    # Subdirectories within implicit "py_*" folder disallowed.
                    if ext == ".py" and os.path.isfile(path):
                        sub.add(name)

        self.__all__ = list(sub) 

    def __getattr__(self, name):
        logger.debug("Getting attribute %s from virtual module %s...", name, self.__name__)

        # Don't try to load anything if we've previously determined that there
        # is no module going by this name.
        if name not in self.__all__:
            raise AttributeError(name)

        try:
            module = self.__loader__.load_module("%s.%s" % (self._fullname, name))
            setattr(self, name, module)
            return module
        except TlImportError:
            raise AttributeError(name)


class TlImport(object):
    """Represents the importer."""

    def __init__(self):
        super().__init__()
        logger.debug("reading $TL_IMPORT_LIBS")
        libs = os.environ.get("TL_IMPORT_LIBS")
        logger.debug(" -> %s", libs)
        if not libs:
            raise TlImportError("$TL_IMPORT_LIBS environment variable is missing or empty")
        libs = list(filter(bool, libs.split(":")))
        if not libs:
            raise TlImportError("$TL_IMPORT_LIBS has no entries in it")
        self._roots = {}
        for lib in libs:
            split = lib.split("=", maxsplit=1)
            if len(split) != 2:
                raise TlImportError("Missing \"=\" in $TL_IMPORT_LIBS entry \"%s\"" % (split[0],))
            name, root = split
            name = name.strip()
            if not re.match(r"[a-zA-Z][a-zA-Z0-9_]*", name):
                raise TlImportError("$TL_IMPORT_LIBS entry \"%s\" is not a valid Python name" % (name,))
            root = root.strip()
            if not os.path.isdir(root):
                raise TlImportError("$TL_IMPORT_LIBS entry \"%s\" root_directory does not exist: \"%s\"" % (name, root))
            self._roots[name] = root
            logger.debug(" -> %s = \"%s\"", name, root)

    def _fullname2path(self, fullname):
        """Converts an import fullname to a two-tuple consisting of a path and
        a boolean indicating whether the path is real or a virtual module must
        be created. If the import is not handled by TlImport, (None, False) is
        returned. If the import is invalid for some reason, this raises
        ImportError."""

        # Match only import paths that start with a known root.
        parts = fullname.split(".")
        try:
            root = self._roots[parts[0]]
        except KeyError:
            return None, False

        # Derive the filename from the import path.
        path = os.path.join(root, *parts[1:])
        if os.path.isdir(path):
            path = os.path.join(path, "__init__.py")
            dummy = not os.path.isfile(path)
            return path, dummy
        elif os.path.isfile(path + ".py"):
            return path + ".py", False
        elif os.path.isdir(os.path.dirname(path)):
            # Try with a py_* directory containing the Python file.
            # Subdirectories within py_* directories are explicitly not allowed
            # in the WoW, so this corner case is not handled.
            dirname, filename = os.path.split(path)
            for subdir in sorted(os.listdir(dirname)):
                if not subdir.startswith("py_"):
                    continue
                implicit_path = os.path.join(dirname, subdir, filename + ".py")
                if os.path.isfile(implicit_path):
                    return implicit_path, False

        raise TlImportError("Module %s was not found at \"%s\" by tl_import" % (fullname, path))

    def find_module(self, fullname, path=None):
        """Returns ourselves when fullname matches <libname>.* and maps to an
        existing or virtual module, raises ImportError when the derived path
        doesn't exist, returns None for non-tl_import imports to defer to
        Python's own loaders."""
        logger.debug("find_module(%r, %r)", fullname, path)
        path, _ = self._fullname2path(fullname)
        if path is None:
            logger.debug(" -> not an import path")
            return None
        else:
            logger.debug(" -> handled by us")
            return self

    def load_module(self, fullname):
        """Loads a module that was previously found by find_module."""
        logger.debug("load_module(%r)", fullname)
        if fullname in sys.modules:
            module = sys.modules[fullname]
            logger.debug(" -> reload: %r", module)
            return module

        path, dummy = self._fullname2path(fullname)
        if path is None:
            raise TlImportError("Module %s is not a tl_import module" % (fullname,))
        logger.debug(" -> maps to \"%s\" (dummy=%s)", path, dummy)

        # For real Python files, defer back to Python's own loader.
        if not dummy:
            path, name = os.path.split(path)
            name, _ = os.path.splitext(name)
            file, filename, data = imp.find_module(name, [path])
            module = imp.load_module(fullname, file, filename, data)
            logger.debug(" -> module loaded: %r", module)
            return module

        # Generate a virtual module with the right path.
        module = TlVirtualModule(self, fullname, path)
        logger.debug(" -> virtual module created: %r", module)
        logger.debug(" -> its submodules are: %r", module.__all__)
        sys.modules[fullname] = module
        return module


# Register tl_import.
sys.meta_path.append(TlImport())
logger.debug("tl_import registered")
