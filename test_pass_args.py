import os
import json
import numpy as np


args = {"val": [50]}
json_object = json.dumps(args)
with open("./jolt_test/input.json", "w") as outfile:
    outfile.write(json_object)
os.chdir("jolt_test")
os.system("cargo run --release prove")
os.system("cargo run --release verify")
os.chdir("/Users/noel/Desktop/Jolt_test")

output = open("./jolt_test/output.json")
out_data = json.load(output)
print(out_data)
