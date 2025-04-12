#!/usr/bin/env python3
import os
import json
import subprocess
import glob
from pathlib import Path
import time
import sys
import platform

# Cores para saída no terminal
class Colors:
    HEADER = '\033[95m'
    OKBLUE = '\033[94m'
    OKGREEN = '\033[92m'
    WARNING = '\033[93m'
    FAIL = '\033[91m'
    ENDC = '\033[0m'
    BOLD = '\033[1m'
    UNDERLINE = '\033[4m'

# Verificar se estamos no Windows
IS_WINDOWS = platform.system() == "Windows"

# Configuração
BINARY_PATH = os.path.join("target", "release", "lz4_messagepack" + (".exe" if IS_WINDOWS else ""))
TEST_DIR = os.path.join("tests", "generated")
OUTPUT_DIR = os.path.join("tests", "results")
FORMATS = ["json", "human", "hex", "binary"]

def setup():
    """Configura as pastas necessárias e compila o binário."""
    os.makedirs(OUTPUT_DIR, exist_ok=True)
    
    # Compilar o binário
    print(f"{Colors.HEADER}Compilando o projeto...{Colors.ENDC}")
    result = subprocess.run(["cargo", "build", "--release"], capture_output=True, text=True)
    if result.returncode != 0:
        print(f"{Colors.FAIL}Falha ao compilar: {result.stderr}{Colors.ENDC}")
        sys.exit(1)
    
    # Verificar se o binário foi gerado
    if not os.path.exists(BINARY_PATH):
        print(f"{Colors.FAIL}Binário não encontrado: {BINARY_PATH}{Colors.ENDC}")
        sys.exit(1)
    
    # Gerar os dados de teste se não existirem
    if not os.path.exists(TEST_DIR) or not os.listdir(TEST_DIR):
        print(f"{Colors.HEADER}Gerando dados de teste...{Colors.ENDC}")
        result = subprocess.run(["cargo", "run", "--bin", "generate_test_data"], capture_output=True, text=True)
        if result.returncode != 0:
            print(f"{Colors.FAIL}Falha ao gerar dados de teste: {result.stderr}{Colors.ENDC}")
            sys.exit(1)

def run_test(input_file, output_format):
    """Executa um teste com o arquivo de entrada e formato especificados."""
    output_file = os.path.join(
        OUTPUT_DIR, 
        f"{os.path.basename(input_file).split('.')[0]}_{output_format}.out"
    )
    
    # Comando para executar o teste
    cmd = [BINARY_PATH, input_file, output_format]
    
    # Executar o teste e medir o tempo
    start_time = time.time()
    if output_format == "binary":
        # Para saída binária, redirecionar para um arquivo
        with open(output_file, "wb") as f:
            result = subprocess.run(cmd, stdout=f, stderr=subprocess.PIPE)
    else:
        # Para outras saídas, capturar e salvar
        result = subprocess.run(cmd, capture_output=True)
        with open(output_file, "wb") as f:
            f.write(result.stdout)
    
    elapsed_time = time.time() - start_time
    
    # Verificar o resultado
    success = result.returncode == 0
    
    # Salvar stderr para depuração
    stderr_file = output_file + ".stderr"
    with open(stderr_file, "wb") as f:
        f.write(result.stderr)
    
    return {
        "success": success,
        "time": elapsed_time,
        "output_file": output_file,
        "stderr_file": stderr_file,
        "stderr": result.stderr.decode('utf-8', errors='replace') if result.stderr else ""
    }

def validate_output(output_file, format_type):
    """Valida se a saída está no formato correto."""
    if not os.path.exists(output_file):
        return False, "Arquivo de saída não encontrado"
    
    try:
        with open(output_file, "rb") as f:
            content = f.read()
        
        if format_type == "json":
            # Verificar se é JSON válido
            json.loads(content)
            return True, "JSON válido"
        elif format_type == "human":
            # Verificar se parece um JSON humanamente legível
            json.loads(content)
            return True, "JSON legível"
        elif format_type == "hex":
            # Verificar se contém apenas hexadecimal
            content_str = content.decode('utf-8', errors='replace').strip()
            is_hex = all(c in "0123456789abcdefABCDEF" for c in content_str)
            return is_hex, "Hexadecimal válido" if is_hex else "Contém caracteres não hexadecimais"
        elif format_type == "binary":
            # Simplesmente verificar se há conteúdo
            return len(content) > 0, f"Conteúdo binário de {len(content)} bytes"
        
        return False, "Formato desconhecido"
    except Exception as e:
        return False, f"Erro na validação: {str(e)}"

def run_all_tests():
    """Executa todos os testes disponíveis."""
    # Encontrar todos os arquivos de teste
    test_files = glob.glob(os.path.join(TEST_DIR, "*.json"))
    
    if not test_files:
        print(f"{Colors.WARNING}Nenhum arquivo de teste encontrado em {TEST_DIR}{Colors.ENDC}")
        return
    
    results = []
    total_tests = len(test_files) * len(FORMATS)
    completed = 0
    success_count = 0
    
    print(f"{Colors.HEADER}Executando {total_tests} testes ({len(test_files)} arquivos x {len(FORMATS)} formatos)...{Colors.ENDC}")
    
    for input_file in test_files:
        test_name = os.path.basename(input_file).split('.')[0]
        print(f"\n{Colors.BOLD}Testando {test_name}:{Colors.ENDC}")
        
        for output_format in FORMATS:
            print(f"  {output_format}: ", end="", flush=True)
            
            # Executar o teste
            test_result = run_test(input_file, output_format)
            completed += 1
            
            # Validar o resultado
            if test_result["success"]:
                valid, message = validate_output(test_result["output_file"], output_format)
                test_result["valid"] = valid
                test_result["validation_message"] = message
                
                if valid:
                    success_count += 1
                    print(f"{Colors.OKGREEN}OK{Colors.ENDC} ({test_result['time']:.3f}s) - {message}")
                else:
                    print(f"{Colors.WARNING}Saída inválida{Colors.ENDC} ({test_result['time']:.3f}s) - {message}")
            else:
                test_result["valid"] = False
                test_result["validation_message"] = "Falha na execução"
                print(f"{Colors.FAIL}Falha{Colors.ENDC} ({test_result['time']:.3f}s) - Erro:")
                for line in test_result["stderr"].split('\n'):
                    if line.strip():
                        print(f"    {line}")
            
            # Adicionar ao resultado
            results.append({
                "test_name": test_name,
                "format": output_format,
                "success": test_result["success"],
                "valid": test_result.get("valid", False),
                "time": test_result["time"],
                "message": test_result.get("validation_message", "")
            })
    
    # Gerar relatório
    print(f"\n{Colors.HEADER}Relatório de testes:{Colors.ENDC}")
    print(f"Total de testes: {total_tests}")
    print(f"Bem-sucedidos: {success_count} ({success_count/total_tests*100:.1f}%)")
    print(f"Falhas: {total_tests - success_count} ({(total_tests - success_count)/total_tests*100:.1f}%)")
    
    # Salvar relatório em JSON
    report_file = os.path.join(OUTPUT_DIR, "test_report.json")
    with open(report_file, "w") as f:
        json.dump({
            "total": total_tests,
            "success": success_count,
            "results": results
        }, f, indent=2)
    
    print(f"\nRelatório salvo em: {report_file}")

if __name__ == "__main__":
    setup()
    run_all_tests() 