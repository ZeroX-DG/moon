import widlparser
import glob

def main():
    widl = widlparser.Parser()

    for idl_file in glob.glob("../idl/HTML*Element.idl"):
        with open(idl_file, "r") as f:
            widl.parse(f.read())
            interface = widl.constructs[0]
            for member in interface.members:
                print(member)
        widl.reset()

main()
