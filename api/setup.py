from distutils import cmd
from setuptools import setup
from pathlib import Path
import os
import sys

HERE = Path(__file__).parent

# manifest.in should copy the protos dir into this directory
PROTO_DIR = Path(os.getenv("CAO_PROTOS_PATH", HERE / ".." / "protos"))


class ProtosCommand(cmd.Command):
    description = "compile the protos"
    user_options = [("protos-dir", None, "Path to the protos directory")]

    def initialize_options(self):
        self.protos_dir = PROTO_DIR

    def finalize_options(self):
        assert os.path.exists(self.protos_dir), (
            "provided protos direcory [%s] does not exist" % self.protos_dir
        )

    def run(self):
        # produce python files from our proto files
        for e in os.listdir(self.protos_dir):
            if ".proto" in e:
                res = os.system(
                    " ".join(
                        [
                            sys.executable,
                            "-m",
                            "grpc_tools.protoc",
                            "-I",
                            str(self.protos_dir),
                            "--python_out",
                            str(HERE / "caoloapi/protos"),
                            "--grpc_python_out",
                            str(HERE / "caoloapi/protos"),
                            str(self.protos_dir / e),
                        ]
                    )
                )
                assert res == 0, f"Failed to compile proto {e} to Python"


setup(
    name="caoloapi",
    package_dir={"": "."},
    install_requires=[
        # Use `poetry` to install the dependencies. This scripts is only used to initialize the protos
    ],
    cmdclass={
        "protos": ProtosCommand,
    },
)
