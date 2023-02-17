import argparse
import random


random.seed(24)


def generate_test_seq_br_c(n: int, is_safe: bool):
    with open(f"test_seq_br_{'safe' if is_safe else 'unsafe'}_{n}.c", "w") as output_c_file:
        output_c_file.write('#include <assert.h>\n')
        output_c_file.write('#include <stdbool.h>\n')
        output_c_file.write('#include <klee.h>\n')
        output_c_file.write('int test(\n')
        for i in range(0, n):
            output_c_file.write(f"\tbool c{i+1}{',' if i != n - 1 else ''}\n")
        output_c_file.write(') {\n')
        output_c_file.write('\tint r = 0;\n')
        for i in range(0, n):
            sign = '+' if is_safe or i % 2 == 1 else '-'
            output_c_file.write(f'\tif (c{i+1}) {{\n')
            output_c_file.write(f'\t\tr {sign}= {i+1};\n')
            output_c_file.write(f'\t}}\n')
        output_c_file.write('\tassert(r >= 0);\n')
        output_c_file.write('\treturn r;\n')
        output_c_file.write('}\n')

        output_c_file.write('int main() {\n')
        for i in range(0, n):
            output_c_file.write(f'\tbool c{i+1};\n')
        for i in range(0, n):
            output_c_file.write(f'\tklee_make_symbolic(&c{i+1}, sizeof(c{i+1}), "c{i+1}");\n')
        output_c_file.write('\treturn test(')
        for i in range(0, n):
            output_c_file.write(f"c{i+1}{', ' if i != n - 1 else ''}")
        output_c_file.write(');\n')
        output_c_file.write('}\n')


def generate_test_seq_br_rust(n: int, is_safe: bool):
    with open(f"test_seq_br_{'safe' if is_safe else 'unsafe'}_{n}.rs", "w") as output_c_file:
        output_c_file.write('fn test(\n')
        for i in range(0, n):
            output_c_file.write(f"\t c{i+1}: bool{',' if i != n - 1 else ''}\n")
        output_c_file.write(') -> i32 {\n')
        for i in range(0, n):
            sign = '' if is_safe or i % 2 == 1 else '-'
            output_c_file.write(f'\tlet r{i+1} = if c{i+1} {{\n')
            output_c_file.write(f'\t\t{sign}{i+1}\n')
            output_c_file.write(f'\t}} else {{\n')
            output_c_file.write(f'\t\t0\n')
            output_c_file.write(f'\t}};\n')
        output_c_file.write('\tlet r = ')
        for i in range(0, n):
            output_c_file.write(f"r{i+1}{' + ' if i != n - 1 else ';'}")
        output_c_file.write('\tassert!(r >= 0);\n')
        output_c_file.write('\treturn r;\n')
        output_c_file.write('}\n')

        output_c_file.write('fn main() {\n')
        output_c_file.write('\ttest(')
        for i in range(0, n):
            output_c_file.write(f"{'false' if random.randint(0, 1) == 0 else 'true'}{', ' if i != n - 1 else ''}")
        output_c_file.write(');\n')
        output_c_file.write('}\n')


def main():
    SAFE = 'safe'
    UNSAFE = 'unsafe'

    RUST = 'rust'
    C = 'c'

    parser = argparse.ArgumentParser(description='Generate test input source code for sequential branches.')
    parser.add_argument('language', choices=[RUST, C])
    parser.add_argument('n', type=int, help='an integer for the number of sequential branches to generate')
    parser.add_argument('safety', choices=[SAFE, UNSAFE])
    args = parser.parse_args()
    assert(args.n >= 0)

    language = args.language
    n = args.n
    is_safe = args.safety == SAFE
    
    if language == C:
        generate_test_seq_br_c(n, is_safe)
    else:
        generate_test_seq_br_rust(n, is_safe)
        

main()
