import os
def main():
    bash = "curl -F extension=text -F data=@SmallFile.txt -F name=smallfile -F key=Test -X POST http://localhost:4041/upload"

    for i in range(5):
            os.system(bash)

if __name__ == "__main__":
    main()