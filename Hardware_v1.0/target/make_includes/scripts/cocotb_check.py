#!/usr/bin/python3
# ************************************************************************
#
# (C) COPYRIGHT 2009 TECHNOLUTION BV, GOUDA NL
# | =======          I                   ==          I    =
# |    I             I                    I          I
# |    I   ===   === I ===  I ===   ===   I  I    I ====  I   ===  I ===
# |    I  /   \ I    I/   I I/   I I   I  I  I    I  I    I  I   I I/   I
# |    I  ===== I    I    I I    I I   I  I  I    I  I    I  I   I I    I
# |    I  \     I    I    I I    I I   I  I  I   /I  \    I  I   I I    I
# |    I   ===   === I    I I    I  ===  ===  === I   ==  I   ===  I    I
# |                 +---------------------------------------------------+
# +----+            |  +++++++++++++++++++++++++++++++++++++++++++++++++|
#      |            |             ++++++++++++++++++++++++++++++++++++++|
#      +------------+                          +++++++++++++++++++++++++|
#                                                         ++++++++++++++|
#              A U T O M A T I O N     T E C H N O L O G Y         +++++|
#
# ************************************************************************
""" check results file from cocotb

    return exit status 1 if result file is invalid, or contains failures
    returns exit status 0 if all is oke

    @author        : Jonathan Hofman (jonathan.hofman@technolution.nl)

"""

# *****************************************************************************
from argparse import ArgumentParser
from xml.etree.ElementTree import ElementTree, ParseError
import os.path

# obtainOptions
def obtainOptions():
    """ Obtain the application options. Currently the application is only
        configured using command line arguments.
    """
    parser = ArgumentParser(description="check cocotb results")
    parser.add_argument('input_file', metavar='file', type=str,
                        help="input result.xml file")
    parser.add_argument('-s', action='store_true', dest='silent', help="silent")
    args = parser.parse_args()

    return args

#################################################################################
## main
#################################################################################
args = obtainOptions()

if not os.path.exists(args.input_file):
    print("[error] input file '{}' does not exist".format(args.input_file))
    exit(1)
try:
    tree = ElementTree()
    tree.parse(args.input_file)
except ParseError:
    print("[error] xml parse error, input file is not valid xml")
    exit(1)

failures = list(tree.iter("failure"))

if len(failures) != 0:
    if not args.silent:
        print("[error] testcase failure found")
    exit(1)
else:
    exit(0)


