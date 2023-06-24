import sys
import argparse
import json
from faker import Faker
from jsonschema import Draft7Validator
import pdb

fake = Faker()


# Define how to generate random data for each type
def generate_from_schema(schema, overwrites={}):
    if "type" not in schema:
        return {}

    type = schema["type"]

    if type == "object":
        obj = {}
        if "properties" in schema:
            for property, subschema in schema["properties"].items():
                if property in overwrites and len(overwrites[property]) > 0:
                    obj[property] = overwrites[property].pop(0)
                else:
                    obj[property] = generate_from_schema(subschema, overwrites)
        return obj
    elif type == "array":
        if "items" in schema:
            return generate_from_schema(schema["items"], overwrites)

    elif type == "string":
        format = schema.get("format")
        if format == "email":
            return fake.email()
        elif format == "uri":
            return fake.uri()
        elif format == "date-time":
            return fake.iso8601()
        return fake.pystr()
    elif type == "number" or type == "integer":
        return fake.pyint()
    elif type == "boolean":
        return fake.pybool()
    else:
        return {}


def main():
    parser = argparse.ArgumentParser(
        description="Generates random responses using GitHub API Commits Schemma."
    )

    parser.add_argument("--num-commits", type=int, default=1, help="Number of commits")
    parser.add_argument(
        "--num-files", type=int, default=1, help="Number of files per commit"
    )

    parser.add_argument(
        "--overwrites",
        type=json.loads,
        default=[],
        help="A Python dictionary with the key/values we want to use to replace the corresponding random values",
    )

    args = parser.parse_args()
    overwrites = dict(args.overwrites)

    # Open the JSON schema file
    with open("commits-endpoint-schema.json") as file:
        schema = json.load(file)

    if "items" in schema:
        data = []
        for _ in range(args.num_commits):
            data.append(generate_from_schema(schema["items"], overwrites))

        # Draft7Validator(schema).validate(data)
        print(json.dumps(data, indent=4))


if __name__ == "__main__":
    main()
